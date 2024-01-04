use futures::StreamExt;
use kube::{
    runtime::{controller::Action, watcher::Config as WatcherConfig, Controller},
    Api, Client, CustomResource,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tracing::{error, info, instrument};

use crate::{
    auth::handle_auth,
    gateway::{handle_http_route, handle_reference_grant},
    Error, Metrics, Network, Result, State,
};

pub static KUPO_PORT_FINALIZER: &str = "kupoports.demeter.run";

struct Context {
    pub client: Client,
    pub metrics: Metrics,
}
impl Context {
    pub fn new(client: Client, metrics: Metrics) -> Self {
        Self { client, metrics }
    }
}

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(
    kind = "KupoPort",
    group = "demeter.run",
    version = "v1alpha1",
    shortname = "kpts",
    namespaced
)]
#[kube(status = "KupoPortStatus")]
#[kube(printcolumn = r#"
        {"name": "Network", "jsonPath": ".spec.network", "type": "string"},
        {"name": "Pruned", "jsonPath": ".spec.pruneUtxo", "type": "boolean"},
        {"name": "Throughput Tier", "jsonPath":".spec.throughputTier", "type": "string"}, 
        {"name": "Endpoint URL", "jsonPath": ".status.endpointUrl", "type": "string"},
        {"name": "Authorization", "jsonPath": ".spec.authorization", "type": "boolean"},
        {"name": "Auth Token", "jsonPath": ".status.authToken", "type": "string"}
    "#)]
#[serde(rename_all = "camelCase")]
pub struct KupoPortSpec {
    pub operator_version: String,
    pub network: Network,
    pub prune_utxo: bool,
    // throughput should be 0, 1, 2
    pub throughput_tier: String,
    pub authorization: bool,
}

#[derive(Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct KupoPortStatus {
    pub endpoint_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
}

async fn reconcile(crd: Arc<KupoPort>, ctx: Arc<Context>) -> Result<Action> {
    handle_reference_grant(ctx.client.clone(), &crd).await?;
    handle_http_route(ctx.client.clone(), &crd).await?;
    handle_auth(ctx.client.clone(), &crd).await?;

    Ok(Action::await_change())
}

fn error_policy(crd: Arc<KupoPort>, err: &Error, ctx: Arc<Context>) -> Action {
    error!(error = err.to_string(), "reconcile failed");
    ctx.metrics.reconcile_failure(&crd, err);
    Action::requeue(Duration::from_secs(5))
}

#[instrument("controller run", skip_all)]
pub async fn run(state: Arc<State>) -> Result<(), Error> {
    info!("listening crds running");

    let client = Client::try_default().await?;
    let crds = Api::<KupoPort>::all(client.clone());
    let ctx = Context::new(client, state.metrics.clone());

    Controller::new(crds, WatcherConfig::default().any_semantic())
        .shutdown_on_signal()
        .run(reconcile, error_policy, Arc::new(ctx))
        .for_each(|_| futures::future::ready(()))
        .await;

    Ok(())
}
