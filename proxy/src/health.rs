use async_trait::async_trait;
use pingora::{server::ShutdownWatch, services::background::BackgroundService};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::{Config, State};

#[derive(Serialize, Deserialize, Debug)]
pub struct KupoHealthCheckResponse {
    pub connection_status: ConnectionStatus,
    pub most_recent_checkpoint: u64,
    pub most_recent_node_tip: u64,
    pub configuration: Configuration,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Configuration {
    pub indexes: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
}

pub struct HealthBackgroundService {
    state: Arc<State>,
    config: Arc<Config>,
}
impl HealthBackgroundService {
    pub fn new(state: Arc<State>, config: Arc<Config>) -> Self {
        Self { state, config }
    }

    async fn get_health(&self) -> bool {
        let client = match reqwest::Client::builder().build() {
            Ok(client) => client,
            Err(err) => {
                warn!(error = err.to_string(), "Failed to build reqwest client");
                return false;
            }
        };

        let response = match client
            .get(format!("{}/health", self.config.instance(true)))
            .header("Accept", "application/json")
            .send()
            .await
        {
            Ok(response) => response,
            Err(err) => {
                warn!(error = err.to_string(), "Failed to perform health request");
                return false;
            }
        };

        let status = response.status();
        if status.is_client_error() || status.is_server_error() {
            error!(status = status.to_string(), "Health request failed");
            return false;
        }

        let Ok(parsed) = response.json::<KupoHealthCheckResponse>().await else {
            warn!("Failed to deserialize health check response");
            return false;
        };

        parsed.connection_status == ConnectionStatus::Connected
    }

    async fn update_health(&self) {
        let current_health = *self.state.upstream_health.read().await;

        let new_health = self.get_health().await;

        match (current_health, new_health) {
            (false, true) => info!("Upstream is now healthy, ready to proxy requests."),
            (true, false) => warn!("Upstream is now deamed unhealthy."),
            _ => {}
        }

        *self.state.upstream_health.write().await = new_health;
    }
}

#[async_trait]
impl BackgroundService for HealthBackgroundService {
    async fn start(&self, mut _shutdown: ShutdownWatch) {
        loop {
            self.update_health().await;
            tokio::time::sleep(self.config.health_poll_interval).await;
        }
    }
}
