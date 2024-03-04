use std::default;

use kube::{
    api::{Patch, PatchParams, PostParams},
    core::{DynamicObject, ObjectMeta},
    discovery::ApiResource,
    Api, Client,
};
use serde_json::json;

use crate::{get_config, Network};

pub fn kong_consumer() -> ApiResource {
    ApiResource {
        group: "configuration.konghq.com".into(),
        version: "v1".into(),
        api_version: "configuration.konghq.com/v1".into(),
        kind: "KongConsumer".into(),
        plural: "kongconsumers".into(),
    }
}

pub async fn get_resource(
    client: Client,
    namespace: &str,
    api_resource: &ApiResource,
    name: &str,
) -> Result<Option<DynamicObject>, kube::Error> {
    let api: Api<DynamicObject> = Api::namespaced_with(client, namespace, api_resource);

    api.get_opt(name).await
}

pub async fn create_resource(
    client: Client,
    namespace: &str,
    api_resource: ApiResource,
    metadata: ObjectMeta,
    data: serde_json::Value,
) -> Result<(), kube::Error> {
    let api: Api<DynamicObject> = Api::namespaced_with(client, namespace, &api_resource);

    let post_params = PostParams::default();

    let mut dynamic = DynamicObject::new("", &api_resource);
    dynamic.data = data;
    dynamic.metadata = metadata;
    api.create(&post_params, &dynamic).await?;
    Ok(())
}

pub async fn patch_resource(
    client: Client,
    namespace: &str,
    api_resource: ApiResource,
    name: &str,
    payload: serde_json::Value,
) -> Result<(), kube::Error> {
    let api: Api<DynamicObject> = Api::namespaced_with(client, namespace, &api_resource);

    let patch_params = PatchParams::default();
    let patch_payload = Patch::Merge(payload);

    api.patch(name, &patch_params, &patch_payload).await?;

    Ok(())
}

pub async fn patch_resource_status(
    client: Client,
    namespace: &str,
    api_resource: ApiResource,
    name: &str,
    payload: serde_json::Value,
) -> Result<(), kube::Error> {
    let api: Api<DynamicObject> = Api::namespaced_with(client, namespace, &api_resource);

    let status = json!({ "status": payload });
    let patch_params = PatchParams::default();
    let patch_payload = Patch::Merge(status);

    api.patch_status(name, &patch_params, &patch_payload)
        .await?;
    Ok(())
}

pub fn build_hostname(
    network: &Network,
    key: &str,
    kupo_version: &Option<String>,
) -> (String, String) {
    let config = get_config();
    let ingress_class = &config.ingress_class;
    let dns_zone = &config.dns_zone;
    let version = kupo_version
        .clone()
        .unwrap_or(config.default_kupo_version.to_string());

    let hostname = format!("{network}-v{version}.{ingress_class}.{dns_zone}");
    let hostname_key = format!("{key}.{network}-v{version}.{ingress_class}.{dns_zone}");

    (hostname, hostname_key)
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn test_build_hostname() {
        env::set_var("METRICS_DELAY", "30");
        env::set_var("PROMETHEUS_URL", "prometheus_url");
        env::set_var("DCU_PER_REQUEST_MAINNET", "3");
        env::set_var("DCU_PER_REQUEST_PREPROD", "3");
        env::set_var("DCU_PER_REQUEST_PREVIEW", "3");
        env::set_var("DCU_PER_REQUEST_SANCHONET", "3");

        let network = Network::Preprod;
        let key = "fake_key";

        let (hostname, hostname_key) = build_hostname(&network, &key, &None);
        assert_eq!(hostname, String::from("preprod-v2.kupo-m1.demeter.run"));
        assert_eq!(
            hostname_key,
            String::from("fake_key.preprod-v2.kupo-m1.demeter.run")
        );

        let (hostname, hostname_key) = build_hostname(&network, &key, &Some("3".to_owned()));
        assert_eq!(hostname, String::from("preprod-v3.kupo-m1.demeter.run"));
        assert_eq!(
            hostname_key,
            String::from("fake_key.preprod-v3.kupo-m1.demeter.run")
        );
    }
}
