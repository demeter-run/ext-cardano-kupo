use crate::{
    auth::handle_auth,
    gateway::{handle_http_route, handle_reference_grant},
    Config, Error, Metrics, Result,
};
use futures::StreamExt;
use kube::{
    runtime::{
        controller::Action,
        finalizer::{finalizer, Event},
        watcher::Config as WatcherConfig,
        Controller,
    },
    Api, Client, CustomResource, ResourceExt,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    sync::Arc,
    time::Duration,
};
use tracing::error;

pub static KUPO_PORT_FINALIZER: &str = "kupoports.demeter.run";

struct Context {
    pub client: Client,
    pub metrics: Metrics,
    pub config: Config,
}
impl Context {
    pub fn new(client: Client, metrics: Metrics, config: Config) -> Self {
        Self {
            client,
            metrics,
            config,
        }
    }
}
#[derive(Clone, Default)]
pub struct State {
    registry: prometheus::Registry,
}
impl State {
    pub fn metrics(&self) -> Vec<prometheus::proto::MetricFamily> {
        self.registry.gather()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub enum Network {
    #[serde(rename = "mainnet")]
    Mainnet,
    #[serde(rename = "preprod")]
    Preprod,
    #[serde(rename = "preview")]
    Preview,
    #[serde(rename = "sanchonet")]
    Sanchonet,
}

impl Display for Network {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Network::Mainnet => write!(f, "mainnet"),
            Network::Preprod => write!(f, "preprod"),
            Network::Preview => write!(f, "preview"),
            Network::Sanchonet => write!(f, "sanchonet"),
        }
    }
}

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(
    kind = "KupoPort",
    group = "demeter.run",
    version = "v1alpha1",
    shortname = "kp",
    namespaced
)]
#[kube(status = "KupoPortStatus")]
#[kube(
    printcolumn = r#"{"name":"Network", "jsonPath": ".spec.network", "type": "string"},
    {"name": "Pruned", "jsonPath": ".spec.pruneUtxo", "type": "boolean"},
    {"name": "Throughput Tier", "jsonPath":"spec.throughputTier", "type": "string"}, 
    {"name": "Endpoint URL", "jsonPath": ".status.endpointUrl",  "type": "string"},
    {"name": "Auth Token", "jsonPath": ".status.authToken", "type": "string"}"#
)]
#[serde(rename_all = "camelCase")]
pub struct KupoPortSpec {
    pub operator_version: String,
    pub network: Network,
    pub prune_utxo: bool,
    // throughput should be 0, 1, 2
    pub throughput_tier: String,
}

#[derive(Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct KupoPortStatus {
    pub endpoint_url: String,
    pub auth_token: String,
}

impl KupoPort {
    async fn reconcile(&self, ctx: Arc<Context>) -> Result<Action> {
        let client = ctx.client.clone();
        let namespace = self.namespace().unwrap();

        let private_dns_service_name =
            build_private_dns_service_name(&self.spec.network, self.spec.prune_utxo);

        handle_reference_grant(client.clone(), &namespace, self, &private_dns_service_name).await?;
        handle_http_route(client.clone(), &namespace, self, &private_dns_service_name).await?;
        handle_auth(client.clone(), &namespace, self).await?;
        Ok(Action::requeue(Duration::from_secs(5 * 60)))
    }

    async fn cleanup(&self, ctx: Arc<Context>) -> Result<Action> {
        Ok(Action::await_change())
    }
}

fn build_private_dns_service_name(network: &Network, prune_utxo: bool) -> String {
    if prune_utxo {
        return format!("kupo-{}-pruned", network);
    }
    format!("kupo-{}", network)
}

async fn reconcile(crd: Arc<KupoPort>, ctx: Arc<Context>) -> Result<Action> {
    // let url = match crd.spec.network {
    //     Network::Mainnet => &ctx.config.kupo_url_mainnet,
    //     Network::Preprod => &ctx.config.kupo_url_preprod,
    //     Network::Preview => &ctx.config.kupo_url_preview,
    // };

    let ns = crd.namespace().unwrap();
    let crds: Api<KupoPort> = Api::namespaced(ctx.client.clone(), &ns);

    finalizer(&crds, KUPO_PORT_FINALIZER, crd, |event| async {
        match event {
            Event::Apply(crd) => crd.reconcile(ctx.clone()).await,
            Event::Cleanup(crd) => crd.cleanup(ctx.clone()).await,
        }
    })
    .await
    .map_err(|e| Error::FinalizerError(Box::new(e)))
}

fn error_policy(crd: Arc<KupoPort>, err: &Error, ctx: Arc<Context>) -> Action {
    error!("reconcile failed: {:?}", err);
    ctx.metrics.reconcile_failure(&crd, err);
    Action::requeue(Duration::from_secs(5))
}

pub async fn run(state: State, config: Config) -> Result<(), Error> {
    let client = Client::try_default().await?;
    let crds = Api::<KupoPort>::all(client.clone());
    let metrics = Metrics::default().register(&state.registry).unwrap();
    let ctx = Context::new(client, metrics, config);

    Controller::new(crds, WatcherConfig::default().any_semantic())
        .shutdown_on_signal()
        .run(reconcile, error_policy, Arc::new(ctx))
        .for_each(|_| futures::future::ready(()))
        .await;

    Ok(())
}
