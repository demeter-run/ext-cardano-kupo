use kube::ResourceExt;
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

    loop {
        sleep(config.metrics_delay)
    }
}
