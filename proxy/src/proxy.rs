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

use crate::config::Config;
use crate::{Consumer, State};

static DMTR_API_KEY: &str = "dmtr-api-key";

pub struct KupoProxy {
    state: Arc<State>,
    config: Arc<Config>,
    host_regex: Regex,
}
impl KupoProxy {
    pub fn new(state: Arc<State>, config: Arc<Config>) -> Self {
        let host_regex =
            Regex::new(r"(dmtr_[\w\d-]+)?\.?([\w]+)-([\w\d]+)\.kupo-([\w\d]+).+").unwrap();

        Self {
            state,
            config,
            host_regex,
        }
    }

    async fn limiter(&self, consumer: &Consumer) -> Result<bool> {
        let tiers = self.state.tiers.read().await.clone();
        let tier = tiers.get(&consumer.tier);

        if tier.is_none() {
            return Ok(true);
        }

        let tier = tier.unwrap();

        let mut rate_limiter_map = self.state.limiter.lock().await;
        let rate_limiter = match rate_limiter_map.get(&consumer.key) {
            None => {
                let limiter = Rate::new(tier.rate_interval);
                rate_limiter_map.insert(consumer.key.clone(), limiter);
                rate_limiter_map.get(&consumer.key).unwrap()
            }
            Some(limiter) => limiter,
        };

        if rate_limiter.observe(&consumer.key, 1) > tier.rate_limit {
            return Ok(true);
        }

        Ok(false)
    }
}

#[derive(Debug, Default)]
pub struct Context {
    instance: String,
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

        let host = session
            .get_header("host")
            .map(|v| v.to_str().unwrap())
            .unwrap();

        let captures = self.host_regex.captures(host).unwrap();
        let network = captures.get(2).unwrap().as_str().to_string();
        let version = captures.get(3).unwrap().as_str().to_string();

        let mut key = session
            .get_header(DMTR_API_KEY)
            .map(|v| v.to_str().unwrap())
            .unwrap_or_default();
        if let Some(m) = captures.get(1) {
            key = m.as_str();
        }

        let consumer = state.get_consumer(&network, &version, key).await;
        if consumer.is_none() {
            return Err(pingora::Error::new(pingora::ErrorType::HTTPStatus(401)));
        }

        let instance = format!(
            "kupo-{network}-pruned.{}:{}",
            self.config.kupo_dns, self.config.kupo_port
        );
        *ctx = Context { instance };

        if self.limiter(&consumer.unwrap()).await? {
            let header = ResponseHeader::build(429, None).unwrap();
            session.set_keepalive(None);
            session.write_response_header(Box::new(header)).await?;
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
}
