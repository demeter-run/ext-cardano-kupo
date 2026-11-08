use crate::{Consumer, State};
use async_trait::async_trait;
use futures_util::TryStreamExt;
use kube::{
    runtime::watcher::{self, Config as ConfigWatcher, Event},
    Api, Client, ResourceExt,
};
use operator::KupoPort;
use pingora::{server::ShutdownWatch, services::background::BackgroundService};
use std::{collections::HashMap, sync::Arc};
use tokio::pin;
use tracing::{error, info};

pub struct AuthBackgroundService {
    state: Arc<State>,
}
impl AuthBackgroundService {
    pub fn new(state: Arc<State>) -> Self {
        Self { state }
    }
}

#[async_trait]
impl BackgroundService for AuthBackgroundService {
    async fn start(&self, mut _shutdown: ShutdownWatch) {
        let client = Client::try_default()
            .await
            .expect("failed to create kube client");

        let api = Api::<KupoPort>::all(client.clone());
        let stream = watcher::watcher(api.clone(), ConfigWatcher::default());
        pin!(stream);

        let mut relist = None;
        loop {
            let result = stream.try_next().await;
            match result {
                Ok(Some(event)) => apply_event(&self.state, &mut relist, event).await,
                // Empty response from stream. Should never happen.
                Ok(None) => {
                    error!("auth: Empty response from watcher.");
                    continue;
                }
                // Unexpected error when streaming CRDs.
                Err(err) => match err {
                    watcher::Error::WatchError(status) => {
                        error!(
                            code = status.code,
                            reason = ?status.reason,
                            message = ?status.message,
                            "auth: Watch error received from API server"
                        );
                        continue;
                    }
                    other => {
                        // For other errors, wait a bit and retry (self-healing).
                        error!(error = %other, "auth: Watch failed, retrying");
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        continue;
                    }
                },
            }
        }
    }
}

/// Applies one watcher event to the consumer table. A re-list (`Init`,
/// `InitApply`..., `InitDone`) collects ports into `relist` and swaps them in
/// whole at `InitDone`, so ports deleted while the watch was down drop out and
/// the old table keeps serving until the list completes.
async fn apply_event(
    state: &State,
    relist: &mut Option<HashMap<String, Consumer>>,
    event: Event<KupoPort>,
) {
    match event {
        // Stream restart, also run on startup.
        Event::Init => {
            info!("auth: Watcher restarted, listing consumers");
            *relist = Some(HashMap::new());
        }
        Event::InitApply(crd) => match (&crd.status, relist.as_mut()) {
            (Some(_), Some(consumers)) => {
                let consumer = Consumer::from(&crd);
                consumers.insert(consumer.key.clone(), consumer);
            }
            (Some(_), None) => error!("auth: Listed port outside a re-list: {}", crd.name_any()),
            (None, _) => info!("auth: Listed port without status: {}", crd.name_any()),
        },
        Event::InitDone => {
            info!("auth: Watcher re-list done, reseting consumers");
            let consumers = relist.take().unwrap_or_default();
            *state.consumers.write().await = consumers;
            state.limiter.write().await.clear();
        }
        // New port created or updated.
        Event::Apply(crd) => match crd.status {
            Some(_) => {
                info!("auth: Updating consumer: {}", crd.name_any());
                let consumer = Consumer::from(&crd);
                state.limiter.write().await.remove(&consumer.key);
                state
                    .consumers
                    .write()
                    .await
                    .insert(consumer.key.clone(), consumer);
            }
            None => {
                // New ports are created without status. When the status is added, a new
                // Apply event is triggered.
                info!("auth: New port created: {}", crd.name_any());
            }
        },
        // Port deleted.
        Event::Delete(crd) => {
            info!(
                "auth: Port deleted, removing from state: {}",
                crd.name_any()
            );
            if crd.status.is_none() {
                return;
            }
            let consumer = Consumer::from(&crd);
            state.consumers.write().await.remove(&consumer.key);
            state.limiter.write().await.remove(&consumer.key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use operator::{KupoPortSpec, KupoPortStatus};

    fn port(name: &str, key: Option<&str>) -> KupoPort {
        let mut port = KupoPort::new(
            name,
            KupoPortSpec {
                operator_version: None,
                network: "preprod".into(),
                prune_utxo: false,
                throughput_tier: "0".into(),
                kupo_version: None,
                auth_token: None,
            },
        );
        port.metadata.namespace = Some("prj-test".into());
        port.status = key.map(|key| KupoPortStatus {
            endpoint_url: String::new(),
            authenticated_endpoint_url: None,
            auth_token: key.into(),
        });
        port
    }

    async fn keys(state: &State) -> Vec<String> {
        let mut keys: Vec<_> = state.consumers.read().await.keys().cloned().collect();
        keys.sort();
        keys
    }

    // One test: State registers its metrics globally, so it can be built once.
    #[tokio::test]
    async fn watcher_events_maintain_consumers() {
        let state = State::default();
        let mut relist = None;

        // Startup list, including a port the operator has not reconciled yet.
        for event in [
            Event::Init,
            Event::InitApply(port("a", Some("key-a"))),
            Event::InitApply(port("pending", None)),
            Event::InitApply(port("b", Some("key-b"))),
        ] {
            apply_event(&state, &mut relist, event).await;
        }
        assert!(keys(&state).await.is_empty(), "applied before InitDone");
        apply_event(&state, &mut relist, Event::InitDone).await;
        assert_eq!(keys(&state).await, ["key-a", "key-b"]);

        apply_event(&state, &mut relist, Event::Apply(port("c", Some("key-c")))).await;
        apply_event(&state, &mut relist, Event::Apply(port("d", None))).await;
        apply_event(&state, &mut relist, Event::Delete(port("a", Some("key-a")))).await;
        apply_event(&state, &mut relist, Event::Delete(port("d", None))).await;
        assert_eq!(keys(&state).await, ["key-b", "key-c"]);

        // A re-list replaces the table only once complete: "b" was deleted
        // while the watch was down.
        apply_event(&state, &mut relist, Event::Init).await;
        apply_event(
            &state,
            &mut relist,
            Event::InitApply(port("c", Some("key-c"))),
        )
        .await;
        assert_eq!(keys(&state).await, ["key-b", "key-c"]);
        apply_event(&state, &mut relist, Event::InitDone).await;
        assert_eq!(keys(&state).await, ["key-c"]);
    }
}
