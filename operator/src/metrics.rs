use kube::{api::ListParams, Api, ResourceExt};
use prometheus::{opts, IntCounterVec, Registry};
use std::{sync::Arc, thread::sleep};

use crate::{get_config, Error, KupoPort, State};

#[derive(Clone)]
pub struct Metrics {
    pub dcu: IntCounterVec,
    pub failures: IntCounterVec,
}

impl Default for Metrics {
    fn default() -> Self {
        let dcu = IntCounterVec::new(
            opts!("dmtr_consumed_dcus", "quantity of dcu consumed",),
            &["project", "service", "service_type", "tenancy"],
        )
        .unwrap();

        let failures = IntCounterVec::new(
            opts!(
                "crd_controller_reconciliation_errors_total",
                "reconciliation errors",
            ),
            &["instance", "error"],
        )
        .unwrap();

        Metrics { dcu, failures }
    }
}

impl Metrics {
    pub fn register(self, registry: &Registry) -> Result<Self, prometheus::Error> {
        registry.register(Box::new(self.failures.clone()))?;
        registry.register(Box::new(self.dcu.clone()))?;

        Ok(self)
    }

    pub fn reconcile_failure(&self, crd: &KupoPort, e: &Error) {
        self.failures
            .with_label_values(&[crd.name_any().as_ref(), e.metric_label().as_ref()])
            .inc()
    }

    pub fn count_dcu_consumed(&self) {
        self.dcu
            .with_label_values(&[
                "project".into(),
                "service",
                "service_type",
                "tenancy".into(),
            ])
            .inc_by(1);
    }
}

pub async fn run_metrics_collector(_state: Arc<State>) -> Result<(), Error> {
    let config = get_config();

    let kube_client = kube::Client::try_default().await?;
    let http_client = reqwest::Client::builder().build().unwrap();

    let crds = Api::<KupoPort>::all(kube_client.clone());
    let params = ListParams::default();

    loop {
        let object_list = crds.list(&params).await?;
        for kupo in object_list.items.iter() {
            let ns = kupo.namespace();
            if ns.is_none() {
                continue;
            }
            let ns = ns.unwrap();

            let query = format!(
                "sum by (route) (kong_http_requests_total{{service='kupo-v1-ingress-kong-proxy', route=~'.*{ns}.*'}})"
            );
            let result = http_client
                .get(format!("{}/query?query={query}", config.prometheus_url))
                .send()
                .await;

            if result.is_err() {
                // TODO send error to prometheus
                continue;
            }

            let response = result.unwrap();
            let status = response.status();
            if status.is_client_error() || status.is_server_error() {
                // TODO send error to prometheus
                continue;
            }

            let value = response.json::<serde_json::Value>().await.unwrap();
            println!("{value}");
        }

        sleep(config.metrics_delay)
    }
}
