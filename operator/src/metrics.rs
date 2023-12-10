use crate::{Error, KupoPort};
use kube::ResourceExt;
use prometheus::{opts, IntCounterVec, Registry};

#[derive(Clone)]
pub struct Metrics {
    pub users_created: IntCounterVec,
    pub users_deactivated: IntCounterVec,
    pub failures: IntCounterVec,
}

impl Default for Metrics {
    fn default() -> Self {
        let users_created = IntCounterVec::new(
            opts!(
                "crd_controller_users_created_total",
                "total of users created in dbsync",
            ),
            &["username"],
        )
        .unwrap();

        let users_deactivated = IntCounterVec::new(
            opts!(
                "crd_controller_users_deactivated_total",
                "total of users deactivated in dbsync",
            ),
            &["username"],
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

        Metrics {
            users_created,
            users_deactivated,
            failures,
        }
    }
}

impl Metrics {
    pub fn register(self, registry: &Registry) -> Result<Self, prometheus::Error> {
        registry.register(Box::new(self.failures.clone()))?;
        registry.register(Box::new(self.users_created.clone()))?;
        registry.register(Box::new(self.users_deactivated.clone()))?;
        Ok(self)
    }

    pub fn reconcile_failure(&self, crd: &KupoPort, e: &Error) {
        self.failures
            .with_label_values(&[crd.name_any().as_ref(), e.metric_label().as_ref()])
            .inc()
    }

    pub fn count_user_created(&self, username: &str) {
        self.users_created.with_label_values(&[username]).inc();
    }

    pub fn count_user_deactivated(&self, username: &str) {
        self.users_deactivated.with_label_values(&[username]).inc();
    }
}
