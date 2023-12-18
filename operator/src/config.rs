use lazy_static::lazy_static;
use std::{env, time::Duration};

lazy_static! {
    static ref CONTROLLER_CONFIG: Config = Config::from_env();
}

pub fn get_config() -> &'static Config {
    &CONTROLLER_CONFIG
}

#[derive(Debug, Clone)]
pub struct Config {
    pub dns_zone: String,
    pub namespace: String,
    pub ingress_class: String,
    pub api_key_salt: String,
    pub http_port: String,
    pub metrics_delay: Duration,
}

impl Config {
    pub fn from_env() -> Self {
        let metrics_delay = Duration::from_secs(
            std::env::var("METRICS_DELAY")
                .unwrap_or("30".into())
                .parse::<u64>()
                .unwrap(),
        );

        Self {
            dns_zone: env::var("DNS_ZONE").unwrap_or("demeter.run".into()),
            namespace: env::var("NAMESPACE").unwrap_or("ftr-kupo-v1".into()),
            ingress_class: env::var("INGRESS_CLASS").unwrap_or("kupo-v1".into()),
            api_key_salt: env::var("API_KEY_SALT").unwrap_or("kupo-salt".into()),
            http_port: "1442".into(),
            metrics_delay,
        }
    }
}
