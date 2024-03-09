use std::{collections::HashMap, fmt::Display, sync::Arc, time::Duration};

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
use serde::{Deserialize, Deserializer};
use tiers::TierBackgroundService;
use tokio::sync::{Mutex, RwLock};
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
    limiter: Mutex<HashMap<String, Rate>>,
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

#[derive(Debug, Clone, Deserialize)]
pub struct Tier {
    name: String,
    rate_limit: isize,
    #[serde(deserialize_with = "deserialize_duration")]
    rate_interval: Duration,
}
pub fn deserialize_duration<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Duration, D::Error> {
    Ok(Duration::from_secs(Deserialize::deserialize(deserializer)?))
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
