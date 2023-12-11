use kube::{core::ObjectMeta, Client, CustomResourceExt, Resource, ResourceExt};
use serde::Deserialize;
use serde_json::json;

use crate::{
    create_resource, get_auth_name, get_config, get_rate_limit_name, get_resource, http_route,
    patch_resource, patch_resource_status, reference_grant, Error, KupoPort, KupoPortStatus,
};

pub async fn handle_http_route(
    client: Client,
    namespace: &str,
    resource: &KupoPort,
    private_dns_service_name: &str,
) -> Result<(), Error> {
    let name = format!("kupo-{}", resource.name_any());
    let host_name = build_host(&name, &namespace_to_slug(namespace));
    let http_route = http_route();
    let kupo_port = KupoPort::api_resource();

    let result = get_resource(client.clone(), namespace, &http_route, &name).await?;

    let (metadata, data, raw) = route(&name, &host_name, resource, private_dns_service_name)?;

    if result.is_some() {
        println!("Updating http route for {}", resource.name_any());
        patch_resource(client.clone(), namespace, http_route, &name, raw).await?;
    } else {
        println!("Creating http route for {}", resource.name_any());
        create_resource(client.clone(), namespace, http_route, metadata, data).await?;
    }

    let status = KupoPortStatus {
        endpoint_url: Some(format!("https://{}", host_name)),
        ..Default::default()
    };
    patch_resource_status(
        client.clone(),
        namespace,
        kupo_port,
        &resource.name_any(),
        serde_json::to_value(status)?,
    )
    .await?;
    Ok(())
}

pub async fn handle_reference_grant(
    client: Client,
    namespace: &str,
    resource: &KupoPort,
    private_dns_service_name: &str,
) -> Result<(), Error> {
    let name = format!("{}-{}-http", namespace, resource.name_any());
    let reference_grant = reference_grant();
    let config = get_config();

    let result = get_resource(client.clone(), &config.namespace, &reference_grant, &name).await?;

    let (metadata, data, raw) = grant(&name, private_dns_service_name, namespace)?;

    if result.is_some() {
        println!("Updating reference grant for {}", resource.name_any());
        patch_resource(
            client.clone(),
            &config.namespace,
            reference_grant,
            &name,
            raw,
        )
        .await?;
    } else {
        println!("Creating reference grant for {}", resource.name_any());
        // we need to get the deserialized payload
        create_resource(
            client.clone(),
            &config.namespace,
            reference_grant,
            metadata,
            data,
        )
        .await?;
    }
    Ok(())
}

fn build_host(name: &str, project_slug: &str) -> String {
    let config = get_config();

    format!(
        "{}-{}.{}.{}",
        name, project_slug, config.ingress_class, config.dns_zone
    )
}

fn namespace_to_slug(namespace: &str) -> String {
    namespace.split_once('-').unwrap().1.to_string()
}

fn route(
    name: &str,
    hostname: &str,
    owner: &KupoPort,
    private_dns_service_name: &str,
) -> Result<(ObjectMeta, serde_json::Value, serde_json::Value), Error> {
    let config = get_config();
    let http_route = http_route();
    let plugins = format!(
        "{},{}",
        get_auth_name(&owner.name_any()),
        get_rate_limit_name(&owner.spec.throughput_tier)
    );

    let metadata: ObjectMeta = ObjectMeta::deserialize(&json!({
      "name": name,
      "labels": {
        "demeter.run/instance": name,
        "demeter.run/tenancy": "project",
        "demeter.run/kind": "http-route"
      },
      "annotations": {
        "konghq.com/plugins": plugins,
      },
      "ownerReferences": [
        {
          "apiVersion": KupoPort::api_version(&()).to_string(), // @TODO: try to grab this from the owner
          "kind": KupoPort::kind(&()).to_string(), // @TODO: try to grab this from the owner
          "name": owner.name_any(),
          "uid": owner.uid()
        }
      ]
    }))?;

    let data = json!({
      "spec": {
        "hostnames": [hostname],
        "parentRefs": [
          {
            "name": config.ingress_class,
            "namespace": config.namespace
          }
        ],
        "rules": [
          {
            "backendRefs": [
              {
                "kind": "Service",
                "name": private_dns_service_name,
                "port": config.http_port.parse::<i32>()?,
                "namespace": config.namespace
              }
            ]
          }
        ]
      }
    });

    let raw = json!({
      "apiVersion": http_route.api_version,
      "kind": http_route.kind,
      "metadata": metadata,
      "spec": data["spec"]
    });

    Ok((metadata, data, raw))
}

fn grant(
    name: &str,
    private_dns_service_name: &str,
    project_namespace: &str,
) -> Result<(ObjectMeta, serde_json::Value, serde_json::Value), Error> {
    let reference_grant = reference_grant();
    let http_route = http_route();

    let metadata: ObjectMeta = ObjectMeta::deserialize(&json!({
      "name": name,
    }))?;

    let data: serde_json::Value = json!({
      "spec": {
        "from": [
              {
                  "group": http_route.group,
                  "kind": http_route.kind,
                  "namespace": project_namespace,
              },
            ],
        "to": [
            {
                "group": "",
                "kind": "Service",
                "name": private_dns_service_name,
            },
        ],
      }
    });

    let raw = json!({
      "apiVersion": reference_grant.api_version,
      "kind": reference_grant.kind,
      "metadata": metadata,
      "spec": data["spec"]
    });

    Ok((metadata, data, raw))
}
