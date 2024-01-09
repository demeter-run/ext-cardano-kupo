use argon2::Argon2;
use base64::{
    engine::general_purpose::{self},
    Engine as _,
};
use bech32::ToBase32;
use k8s_openapi::{api::core::v1::Secret, apimachinery::pkg::apis::meta::v1::OwnerReference};
use kube::{
    api::{DeleteParams, Patch, PatchParams, PostParams},
    core::ObjectMeta,
    Api, Client, CustomResourceExt, Resource, ResourceExt,
};
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::collections::BTreeMap;
use tracing::info;

use crate::{
    create_resource, delete_resource, get_acl_name, get_auth_name, get_config, get_host_key_name,
    get_resource, kong_consumer, kong_plugin, patch_resource, replace_resource_status,
    Authentication, Error, KupoPort,
};

pub async fn handle_auth(client: Client, crd: &KupoPort) -> Result<(), Error> {
    handle_auth_secret(client.clone(), crd).await?;
    handle_auth_plugin(client.clone(), crd).await?;
    handle_host_key_plugin(client.clone(), crd).await?;

    handle_acl_secret(client.clone(), crd).await?;
    handle_acl_plugin(client.clone(), crd).await?;

    handle_consumer(client.clone(), crd).await?;

    Ok(())
}

async fn handle_auth_secret(client: Client, crd: &KupoPort) -> Result<(), Error> {
    let namespace = crd.namespace().unwrap();
    let name = get_auth_name(&crd.name_any());
    let kupo_port = KupoPort::api_resource();

    let api = Api::<Secret>::namespaced(client.clone(), &namespace);
    let result = api.get_opt(&name).await?;

    let mut status = crd.status.clone().unwrap_or_default();

    match crd.spec.authentication {
        Authentication::ApiKey => {
            let api_key = generate_api_key(&name, &namespace).await?;
            let secret = build_auth_secret(&name, &api_key, crd.clone());

            status.auth_token = Some(api_key);

            if result.is_some() {
                info!(resource = crd.name_any(), "Updating auth secret");
                let patch_params = PatchParams::default();
                api.patch(&name, &patch_params, &Patch::Merge(secret))
                    .await?;
            } else {
                info!(resource = crd.name_any(), "Creating auth secret");
                let post_params = PostParams::default();
                api.create(&post_params, &secret).await?;
            }
        }
        Authentication::None => {
            if result.is_some() {
                info!(resource = crd.name_any(), "Deleting auth secret");
                api.delete(&name, &DeleteParams::default()).await?;
            }
        }
    }

    replace_resource_status(
        client.clone(),
        &namespace,
        kupo_port,
        &crd.name_any(),
        serde_json::to_value(status)?,
    )
    .await?;

    Ok(())
}

async fn handle_auth_plugin(client: Client, crd: &KupoPort) -> Result<(), Error> {
    let namespace = crd.namespace().unwrap();
    let name = get_auth_name(&crd.name_any());
    let kong_plugin = kong_plugin();

    let result = get_resource(client.clone(), &namespace, &kong_plugin, &name).await?;

    match crd.spec.authentication {
        Authentication::ApiKey => {
            let (metadata, data, raw) = build_auth_plugin(crd.clone())?;
            if result.is_some() {
                info!(resource = crd.name_any(), "Updating auth plugin");
                patch_resource(client.clone(), &namespace, kong_plugin, &name, raw).await?;
            } else {
                info!(resource = crd.name_any(), "Creating auth plugin");
                create_resource(client.clone(), &namespace, kong_plugin, metadata, data).await?;
            }
        }
        Authentication::None => {
            if result.is_some() {
                info!(resource = crd.name_any(), "Deleting auth plugin");
                delete_resource(client.clone(), &namespace, kong_plugin, &name).await?;
            }
        }
    }

    Ok(())
}

async fn handle_host_key_plugin(client: Client, crd: &KupoPort) -> Result<(), Error> {
    let namespace = crd.namespace().unwrap();
    let name = get_host_key_name(&crd.name_any());
    let kong_plugin = kong_plugin();

    let result = get_resource(client.clone(), &namespace, &kong_plugin, &name).await?;

    match crd.spec.authentication {
        Authentication::ApiKey => {
            let (metadata, data, raw) = build_host_key_plugin(crd.clone())?;
            if result.is_some() {
                info!(resource = crd.name_any(), "Updating host key plugin");
                patch_resource(client.clone(), &namespace, kong_plugin, &name, raw).await?;
            } else {
                info!(resource = crd.name_any(), "Creating host key plugin");
                create_resource(client.clone(), &namespace, kong_plugin, metadata, data).await?;
            }
        }
        Authentication::None => {
            if result.is_some() {
                info!(resource = crd.name_any(), "Deleting host key plugin");
                delete_resource(client.clone(), &namespace, kong_plugin, &name).await?;
            }
        }
    }

    Ok(())
}

async fn handle_acl_secret(client: Client, crd: &KupoPort) -> Result<(), Error> {
    let namespace = crd.namespace().unwrap();
    let name = get_acl_name(&crd.name_any());

    let api = Api::<Secret>::namespaced(client.clone(), &namespace);

    let result = api.get_opt(&name).await?;

    match crd.spec.authentication {
        Authentication::ApiKey => {
            let secret = build_acl_secret(&name, crd.clone());

            if result.is_some() {
                info!(resource = crd.name_any(), "Updating acl secret");
                let patch_params = PatchParams::default();
                api.patch(&name, &patch_params, &Patch::Merge(secret))
                    .await?;
            } else {
                info!(resource = crd.name_any(), "Creating acl secret");
                let post_params = PostParams::default();
                api.create(&post_params, &secret).await?;
            }
        }
        Authentication::None => {
            if result.is_some() {
                info!(resource = crd.name_any(), "Deleting acl secret");
                api.delete(&name, &DeleteParams::default()).await?;
            }
        }
    }

    Ok(())
}

async fn handle_acl_plugin(client: Client, crd: &KupoPort) -> Result<(), Error> {
    let namespace = crd.namespace().unwrap();
    let name = get_acl_name(&crd.name_any());
    let kong_plugin = kong_plugin();

    let result = get_resource(client.clone(), &namespace, &kong_plugin, &name).await?;
    let (metadata, data, raw) = build_acl_plugin(crd.clone())?;

    match crd.spec.authentication {
        Authentication::ApiKey => {
            if result.is_some() {
                info!(resource = crd.name_any(), "Updating acl plugin");
                patch_resource(client.clone(), &namespace, kong_plugin, &name, raw).await?;
            } else {
                info!(resource = crd.name_any(), "Creating acl plugin");
                create_resource(client.clone(), &namespace, kong_plugin, metadata, data).await?;
            }
        }
        Authentication::None => {
            if result.is_some() {
                info!(resource = crd.name_any(), "Deleting acl plugin");
                delete_resource(client.clone(), &namespace, kong_plugin, &name).await?;
            }
        }
    }

    Ok(())
}

async fn handle_consumer(client: Client, crd: &KupoPort) -> Result<(), Error> {
    let namespace = crd.namespace().unwrap();
    let name = get_auth_name(&crd.name_any());
    let kong_consumer = kong_consumer();

    let result = get_resource(client.clone(), &namespace, &kong_consumer, &name).await?;

    match crd.spec.authentication {
        Authentication::ApiKey => {
            let (metadata, data, raw) = build_consumer(crd.clone())?;
            if result.is_some() {
                info!(resource = crd.name_any(), "Updating consumer");
                patch_resource(client.clone(), &namespace, kong_consumer, &name, raw).await?;
            } else {
                info!(resource = crd.name_any(), "Creating consumer");
                create_resource(client.clone(), &namespace, kong_consumer, metadata, data).await?;
            }
        }
        Authentication::None => {
            if result.is_some() {
                info!(resource = crd.name_any(), "Deleting consumer");
                delete_resource(client.clone(), &namespace, kong_consumer, &name).await?;
            }
        }
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
    let base64 = general_purpose::URL_SAFE_NO_PAD.encode(output);
    let with_bech =
        bech32::encode("dmtr_kupo", base64.to_base32(), bech32::Variant::Bech32).unwrap();

    Ok(with_bech)
}

fn build_auth_secret(name: &str, api_key: &str, owner: KupoPort) -> Secret {
    let mut string_data = BTreeMap::new();
    string_data.insert("key".into(), api_key.into());

    let mut labels = BTreeMap::new();
    labels.insert("konghq.com/credential".into(), "key-auth".into());

    let metadata = ObjectMeta {
        name: Some(name.to_string()),
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

fn build_auth_plugin(owner: KupoPort) -> Result<(ObjectMeta, JsonValue, JsonValue), Error> {
    let kong_plugin = kong_plugin();

    let metadata = ObjectMeta::deserialize(&json!({
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
        "key_names": ["dmtr-api-key"],
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

fn build_host_key_plugin(owner: KupoPort) -> Result<(ObjectMeta, JsonValue, JsonValue), Error> {
    let kong_plugin = kong_plugin();

    let metadata = ObjectMeta::deserialize(&json!({
      "name": get_host_key_name(&owner.name_any()),
      "ownerReferences": [
        {
          "apiVersion": KupoPort::api_version(&()).to_string(),
          "kind": KupoPort::kind(&()).to_string(),
          "name": owner.name_any(),
          "uid": owner.uid()
        }
      ]
    }))?;

    let data = json!({
      "plugin": "key-to-header",
      "config": {}
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

fn build_acl_secret(name: &str, owner: KupoPort) -> Secret {
    let mut string_data = BTreeMap::new();
    string_data.insert("group".into(), owner.name_any());

    let mut labels = BTreeMap::new();
    labels.insert("konghq.com/credential".into(), "acl".into());

    let metadata = ObjectMeta {
        name: Some(name.to_string()),
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

fn build_acl_plugin(owner: KupoPort) -> Result<(ObjectMeta, JsonValue, JsonValue), Error> {
    let kong_plugin = kong_plugin();

    let metadata = ObjectMeta::deserialize(&json!({
      "name": get_acl_name(&owner.name_any()),
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
      "plugin": "acl",
      "config": {
        "allow": [owner.name_any()]
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

fn build_consumer(owner: KupoPort) -> Result<(ObjectMeta, JsonValue, JsonValue), Error> {
    let kong_consumer = kong_consumer();
    let config = get_config();

    let metadata = ObjectMeta::deserialize(&json!({
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
      "credentials": [get_auth_name(&owner.name_any()), get_acl_name(&owner.name_any())]
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
