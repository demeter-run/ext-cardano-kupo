use async_trait::async_trait;
use pingora::http::ResponseHeader;
use pingora::Result;
use pingora::{
    proxy::{ProxyHttp, Session},
    upstreams::peer::HttpPeer,
};
use pingora_limits::rate::Rate;
use regex::Regex;
use std::sync::Arc;
use tracing::info;

use crate::config::Config;
use crate::{Consumer, State, Tier};

static DMTR_API_KEY: &str = "dmtr-api-key";

pub struct KupoProxy {
    state: Arc<State>,
    config: Arc<Config>,
    host_regex: Regex,
}
impl KupoProxy {
    pub fn new(state: Arc<State>, config: Arc<Config>) -> Self {
        let host_regex = Regex::new(r"(dmtr_[\w\d-]+)?\.?.+").unwrap();

        Self {
            state,
            config,
            host_regex,
        }
    }

    async fn has_limiter(&self, consumer: &Consumer) -> bool {
        let rate_limiter_map = self.state.limiter.read().await;
        rate_limiter_map.get(&consumer.key).is_some()
    }

    async fn add_limiter(&self, consumer: &Consumer, tier: &Tier) {
        let rates = tier
            .rates
            .iter()
            .map(|r| (r.clone(), Rate::new(r.interval)))
            .collect();

        self.state
            .limiter
            .write()
            .await
            .insert(consumer.key.clone(), rates);
    }

    async fn limiter(&self, consumer: &Consumer) -> Result<bool> {
        let tiers = self.state.tiers.read().await.clone();
        let tier = tiers.get(&consumer.tier);
        if tier.is_none() {
            return Ok(true);
        }
        let tier = tier.unwrap();

        if !self.has_limiter(consumer).await {
            self.add_limiter(consumer, tier).await;
        }

        let rate_limiter_map = self.state.limiter.read().await;
        let rates = rate_limiter_map.get(&consumer.key).unwrap();

        if rates
            .iter()
            .any(|(t, r)| r.observe(&consumer.key, 1) > t.limit)
        {
            return Ok(true);
        }

        Ok(false)
    }

    async fn respond_health(&self, session: &mut Session, ctx: &mut Context) {
        ctx.is_health_request = true;
        session.set_keepalive(None);
        session.write_response_body("OK".into()).await.unwrap();
        let header = Box::new(ResponseHeader::build(200, None).unwrap());
        session.write_response_header(header).await.unwrap();
    }
}

#[derive(Debug, Default)]
pub struct Context {
    is_health_request: bool,
    instance: String,
    consumer: Consumer,
}

#[async_trait]
impl ProxyHttp for KupoProxy {
    type CTX = Context;
    fn new_ctx(&self) -> Self::CTX {
        Context::default()
    }

    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool>
    where
        Self::CTX: Send + Sync,
    {
        let state = self.state.clone();

        // Check if the request is going to the health endpoint before continuing.
        let path = session.req_header().uri.path();
        if path == self.config.health_endpoint {
            self.respond_health(session, ctx).await;
            return Ok(true);
        }

        // Extract key from host or header.
        let host = session
            .get_header("host")
            .map(|v| v.to_str().unwrap())
            .unwrap();

        let captures = self.host_regex.captures(host).unwrap();
        let mut key = session
            .get_header(DMTR_API_KEY)
            .map(|v| v.to_str().unwrap())
            .unwrap_or_default();
        if let Some(m) = captures.get(1) {
            key = m.as_str();
        }

        let consumer = state.get_consumer(key).await;
        if consumer.is_none() {
            session.respond_error(401).await;
            return Ok(true);
        }

        ctx.consumer = consumer.unwrap();
        ctx.instance = format!(
            "kupo-{}-pruned.{}:{}",
            ctx.consumer.network, self.config.kupo_dns, self.config.kupo_port
        );

        if self.limiter(&ctx.consumer).await? {
            session.respond_error(429).await;
            return Ok(true);
        }

        Ok(false)
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let http_peer = HttpPeer::new(&ctx.instance, false, String::default());
        Ok(Box::new(http_peer))
    }

    async fn logging(
        &self,
        session: &mut Session,
        _e: Option<&pingora::Error>,
        ctx: &mut Self::CTX,
    ) {
        if !ctx.is_health_request {
            let response_code = session
                .response_written()
                .map_or(0, |resp| resp.status.as_u16());

            info!(
                "{} response code: {response_code}",
                self.request_summary(session, ctx)
            );

            self.state.metrics.inc_http_total_request(
                &ctx.consumer,
                &self.config.proxy_namespace,
                &ctx.instance,
                &response_code,
            );
        }
    }
}
