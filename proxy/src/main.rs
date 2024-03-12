use auth::AuthBackgroundService;
use config::Config;
use dotenv::dotenv;
use pingora::{
    server::{configuration::Opt, Server},
    services::background::background_service,
};
use pingora_limits::rate::Rate;
use prometheus::{opts, register_int_counter_vec};
use proxy::KupoProxy;
use serde::{de::Visitor, Deserialize, Deserializer};
use std::{collections::HashMap, fmt::Display, sync::Arc, time::Duration};
use tiers::TierBackgroundService;
use tokio::sync::RwLock;
use tracing::Level;

mod auth;
mod config;
mod proxy;
mod tiers;

fn main() {
    dotenv().ok();

    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let config: Arc<Config> = Arc::default();
    let state: Arc<State> = Arc::default();

    let opt = Opt::default();
    let mut server = Server::new(Some(opt)).unwrap();
    server.bootstrap();

    let auth_background_service = background_service(
        "K8S Auth Service",
        AuthBackgroundService::new(state.clone()),
    );
    server.add_service(auth_background_service);

    let tier_background_service = background_service(
        "K8S Tier Service",
        TierBackgroundService::new(state.clone(), config.clone()),
    );
    server.add_service(tier_background_service);

    let mut kupo_http_proxy = pingora::proxy::http_proxy_service(
        &server.configuration,
        KupoProxy::new(state.clone(), config.clone()),
    );
    kupo_http_proxy
        .add_tls(
            &config.proxy_addr,
            &config.ssl_crt_path,
            &config.ssl_key_path,
        )
        .unwrap();
    server.add_service(kupo_http_proxy);

    let mut prometheus_service = pingora::services::listening::Service::prometheus_http_service();
    prometheus_service.add_tcp(&config.prometheus_addr);
    server.add_service(prometheus_service);

    server.run_forever();
}

#[derive(Default)]
pub struct State {
    consumers: RwLock<HashMap<String, Consumer>>,
    tiers: RwLock<HashMap<String, Tier>>,
    limiter: RwLock<HashMap<String, Vec<(TierRate, Rate)>>>,
    metrics: Metrics,
}
impl State {
    pub async fn get_consumer(&self, network: &str, version: &str, key: &str) -> Option<Consumer> {
        let consumers = self.consumers.read().await.clone();
        let hash_key = format!("{}.{}.{}", network, version, key);
        consumers.get(&hash_key).cloned()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Consumer {
    namespace: String,
    port_name: String,
    tier: String,
    key: String,
}
impl Consumer {
    pub fn new(namespace: String, port_name: String, tier: String, key: String) -> Self {
        Self {
            namespace,
            port_name,
            key,
            tier,
        }
    }
}
impl Display for Consumer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.namespace, self.port_name)
    }
}

#[derive(Debug, Clone)]
pub struct Tier {
    name: String,
    rates: Vec<TierRate>,
}
#[derive(Debug, Clone)]
pub struct TierRate {
    limit: isize,
    interval: Duration,
}

impl<'de> Deserialize<'de> for Tier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(TierVisitor)
    }
}

struct TierVisitor;
impl<'de> Visitor<'de> for TierVisitor {
    type Value = Tier;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("This Visitor expects to receive a map tier struct")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut rates: Vec<TierRate> = Vec::default();

        let mut name = Default::default();

        while let Some(key) = map.next_key::<String>()? {
            if key == "name" {
                name = map.next_value()?;
                continue;
            }

            let rate = match key.as_str() {
                "second" => Ok(TierRate {
                    limit: map.next_value()?,
                    interval: Duration::from_secs(1),
                }),
                "minute" => Ok(TierRate {
                    limit: map.next_value()?,
                    interval: Duration::from_secs(60),
                }),
                "hour" => Ok(TierRate {
                    limit: map.next_value()?,
                    interval: Duration::from_secs(60 * 60),
                }),
                "day" => Ok(TierRate {
                    limit: map.next_value()?,
                    interval: Duration::from_secs(60 * 60 * 24),
                }),
                _ => Err(serde::de::Error::custom("Invalid symbol tier interval")),
            }?;

            rates.push(rate);
        }

        let tier = Tier { name, rates };

        Ok(tier)
    }
}

#[derive(Debug, Clone)]
pub struct Metrics {
    http_total_request: prometheus::IntCounterVec,
}
impl Metrics {
    pub fn new() -> Self {
        let http_total_request = register_int_counter_vec!(
            opts!("kupo_proxy_http_total_request", "Total http request",),
            &["consumer", "namespace", "instance", "status_code",]
        )
        .unwrap();

        Self { http_total_request }
    }

    pub fn inc_http_total_request(
        &self,
        consumer: &Consumer,
        namespace: &str,
        instance: &str,
        status: &u16,
    ) {
        let consumer = &consumer.to_string();

        self.http_total_request
            .with_label_values(&[consumer, namespace, instance, &status.to_string()])
            .inc()
    }
}
impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}
