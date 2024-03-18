use std::{env, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Config {
    pub proxy_addr: String,
    pub proxy_namespace: String,
    pub proxy_tiers_path: PathBuf,
    pub prometheus_addr: String,
    pub ssl_crt_path: String,
    pub ssl_key_path: String,
    pub kupo_port: u16,
    pub kupo_dns: String,
    pub default_kupo_version: String,
}
impl Config {
    pub fn new() -> Self {
        Self {
            proxy_addr: env::var("PROXY_ADDR").expect("PROXY_ADDR must be set"),
            proxy_namespace: env::var("PROXY_NAMESPACE").expect("PROXY_NAMESPACE must be set"),
            proxy_tiers_path: env::var("PROXY_TIERS_PATH")
                .map(|v| v.into())
                .expect("PROXY_TIERS_PATH must be set"),
            prometheus_addr: env::var("PROMETHEUS_ADDR").expect("PROMETHEUS_ADDR must be set"),
            ssl_crt_path: env::var("SSL_CRT_PATH").expect("SSL_CRT_PATH must be set"),
            ssl_key_path: env::var("SSL_KEY_PATH").expect("SSL_KEY_PATH must be set"),
            kupo_port: env::var("KUPO_PORT")
                .expect("KUPO_PORT must be set")
                .parse()
                .expect("KUPO_PORT must a number"),
            kupo_dns: env::var("KUPO_DNS").expect("KUPO_DNS must be set"),
            default_kupo_version: env::var("DEFAULT_KUPO_VERSION").unwrap_or("v2".into()),
        }
    }
}
impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
