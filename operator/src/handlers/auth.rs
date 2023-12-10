use std::collections::BTreeMap;

use argon2::Argon2;
use base64::{
    engine::general_purpose::{self},
    Engine as _,
};
use k8s_openapi::{api::core::v1::Secret, apimachinery::pkg::apis::meta::v1::OwnerReference};
use kube::{
    api::{Patch, PatchParams, PostParams},
    core::ObjectMeta,
    Api, Client, CustomResourceExt, Resource, ResourceExt,
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    create_resource, get_auth_name, get_config, get_resource, kong_consumer, kong_plugin,
    patch_resource, patch_resource_status, Error, KupoPort,
};

pub async fn handle_auth(
    client: Client,
    namespace: &str,
    resource: &KupoPort,
) -> Result<(), Error> {
    handle_secret(client.clone(), namespace, resource).await?;
    handle_auth_plugin(client.clone(), namespace, resource).await?;
    handle_consumer(client.clone(), namespace, resource).await?;
    Ok(())
}

async fn handle_secret(client: Client, namespace: &str, resource: &KupoPort) -> Result<(), Error> {
    let name = get_auth_name(&resource.name_any());
    let api_key = generate_api_key(&name, namespace).await?;
    let kupo_port = KupoPort::api_resource();

    let api = Api::<Secret>::namespaced(client.clone(), namespace);

    let secret = secret(&api_key, resource.clone());
    let result = api.get(&name).await?;

    if result.data.is_some() {
        let patch_params = PatchParams::default();
        api.patch(&name, &patch_params, &Patch::Merge(secret))
            .await?;
    } else {
        let post_params = PostParams::default();
        api.create(&post_params, &secret).await?;
    }

    let status = json!({
      "status": {
        "auth_token": api_key,
      }
    });
    patch_resource_status(client.clone(), namespace, kupo_port, &name, status).await?;
    Ok(())
}

async fn handle_auth_plugin(
    client: Client,
    namespace: &str,
    resource: &KupoPort,
) -> Result<(), Error> {
    let name = get_auth_name(&resource.name_any());
    let kong_plugin = kong_plugin();

    let result = get_resource(client.clone(), namespace, &kong_plugin, &name).await?;
    let (metadata, data, raw) = auth_plugin(resource.clone())?;

    if result.is_some() {
        patch_resource(client.clone(), namespace, kong_plugin, &name, raw).await?;
    } else {
        create_resource(client.clone(), namespace, kong_plugin, metadata, data).await?;
    }
    Ok(())
}

async fn handle_consumer(
    client: Client,
    namespace: &str,
    resource: &KupoPort,
) -> Result<(), Error> {
    let name = get_auth_name(&resource.name_any());
    let kong_consumer = kong_consumer();

    let result = get_resource(client.clone(), namespace, &kong_consumer, &name).await?;
    let (metadata, data, raw) = consumer(resource.clone())?;

    if result.is_some() {
        patch_resource(client.clone(), namespace, kong_consumer, &name, raw).await?;
    } else {
        create_resource(client.clone(), namespace, kong_consumer, metadata, data).await?;
    }
    Ok(())
}

async fn generate_api_key(name: &str, namespace: &str) -> Result<String, Error> {
    let password = format!("{}{}", name, namespace).as_bytes().to_vec();

    let config = get_config();
    let salt = config.api_key_salt.as_bytes();

    let mut output = vec![0; 16];

    let argon2 = Argon2::default();
    let _ = argon2.hash_password_into(password.as_slice(), salt, &mut output);

    // Encode the hash using Bech32
    let with_bech = general_purpose::URL_SAFE_NO_PAD.encode(output);

    Ok(with_bech)
}

fn secret(api_key: &str, owner: KupoPort) -> Secret {
    let mut string_data = BTreeMap::new();
    string_data.insert(String::from("key"), String::from(api_key).to_string());

    let mut labels = BTreeMap::new();
    labels.insert(
        String::from("konghq.com/credential"),
        String::from("key-auth"),
    );

    let metadata = ObjectMeta {
        name: Some(owner.name_any()),
        owner_references: Some(vec![OwnerReference {
            api_version: KupoPort::api_version(&()).to_string(),
            kind: KupoPort::kind(&()).to_string(),
            name: owner.name_any(),
            uid: owner.uid().unwrap(),
            ..Default::default()
        }]),
        labels: Some(labels),
        ..Default::default()
    };

    Secret {
        type_: Some(String::from("Opaque")),
        metadata,
        string_data: Some(string_data),
        ..Default::default()
    }
}

fn auth_plugin(
    owner: KupoPort,
) -> Result<(ObjectMeta, serde_json::Value, serde_json::Value), Error> {
    let kong_plugin = kong_plugin();

    let metadata: ObjectMeta = ObjectMeta::deserialize(&json!({
      "name": get_auth_name(&owner.name_any()),

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
      "plugin": "key-auth",
      "config": {
        "key_names": ["key"],

      }
    });

    let raw = json!({
        "apiVersion": kong_plugin.api_version,
        "kind": kong_plugin.kind,
        "metadata": metadata,
        "plugin": data["plugin"],
        "config": data["config"]
    });

    Ok((metadata, data, raw))
}

fn consumer(owner: KupoPort) -> Result<(ObjectMeta, serde_json::Value, serde_json::Value), Error> {
    let kong_consumer = kong_consumer();
    let config = get_config();

    let metadata: ObjectMeta = ObjectMeta::deserialize(&json!({
      "name": get_auth_name(&owner.name_any()),
      "annotations": {
        "kubernetes.io/ingress.class": config.ingress_class,
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
      "username": owner.name_any(),
      "credentials": [get_auth_name(&owner.name_any())]
    });

    let raw = json!({
        "apiVersion": kong_consumer.api_version,
        "kind": kong_consumer.kind,
        "metadata": metadata,
        "username": data["username"],
        "credentials": data["credentials"]
    });

    Ok((metadata, data, raw))
}
