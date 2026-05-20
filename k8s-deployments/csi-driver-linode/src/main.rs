use std::{collections::BTreeMap, env};
use k8s_openapi::api::storage::v1::StorageClass;
use kube::core::ObjectMeta;
use kube_builder::prelude::*;
use serde::Deserialize;
use serde_norway::Value;

fn main() -> anyhow::Result<()> {
    let sealed_secrets = read_sealed_secrets_from_stdin()?;
    let mut resources: Vec<Vec<serde_json::Value>> = Vec::new();

    if !sealed_secrets.is_empty() {
        resources.push(sealed_secrets);
    }

    let helm_out_str = include_str!(concat!(env!("OUT_DIR"), "/helm-output.yaml"));
    let mut helm_resources: Vec<serde_json::Value> = Vec::new();
    for document in serde_norway::Deserializer::from_str(helm_out_str) {
        let value = Value::deserialize(document)?;
        if value != Value::Null {
            // Skip the Helm-generated Secret — replaced by SealedSecret via sops-seal
            if *value.get("kind").unwrap() == Value::String(String::from("Secret")) {
                continue;
            }
            // Remove the default storage class annotation from any storage
            // classes that are set as default by the Helm chart; the encrypted
            // storage class will be the only default
            if *value.get("kind").unwrap() == Value::String(String::from("StorageClass")) {
                let mut sc: StorageClass = serde_norway::from_value(value).unwrap();
                if sc.metadata.annotations.is_some() {
                    sc.metadata.annotations.as_mut().unwrap().remove("storageclass.kubernetes.io/is-default-class");
                }
                helm_resources.push(serde_json::to_value(sc)?);
            } else {
                let mut json = serde_json::to_value(value)?;
                fix_env_value_conflicts(&mut json);
                helm_resources.push(json);
            }
        }
    }

    for sc in create_encrypted_storage_classes() {
        helm_resources.push(serde_json::to_value(sc)?);
    }

    resources.push(helm_resources);
    println!("{}", serde_json::to_string(&resources)?);
    Ok(())
}

// The Helm chart generates env vars with both `value` and `valueFrom` set, which Kubernetes
// rejects as invalid. Strip `value` from any env entry that also has `valueFrom`.
fn fix_env_value_conflicts(value: &mut serde_json::Value) {
    if let Some(containers) = value
        .pointer_mut("/spec/template/spec/containers")
        .and_then(|v| v.as_array_mut())
    {
        for container in containers {
            if let Some(envs) = container
                .get_mut("env")
                .and_then(|e| e.as_array_mut())
            {
                for env in envs {
                    if let Some(obj) = env.as_object_mut() {
                        if obj.contains_key("valueFrom") {
                            obj.remove("value");
                        }
                    }
                }
            }
        }
    }
}

fn create_encrypted_storage_classes() -> Vec<StorageClass> {
    // WaitForFirstConsumer delays volume provisioning until a pod is scheduled, ensuring
    // the volume is created in the same zone as the pod and avoiding topology mismatches.
    vec![
        StorageClass {
            metadata: ObjectMeta {
                name: Some(String::from("linode-block-storage-encrypted")),
                namespace: Some(String::from("kube-system")),
                ..Default::default()
            },
            parameters: Some(BTreeMap::from([
                                (String::from("linodebs.csi.linode.com/encrypted"), String::from("true"))
            ])),
            allow_volume_expansion: Some(true),
            provisioner: String::from("linodebs.csi.linode.com"),
            volume_binding_mode: Some(String::from("WaitForFirstConsumer")),
            ..Default::default()
        },
        StorageClass {
            metadata: ObjectMeta {
                name: Some(String::from("linode-block-storage-retain-encrypted")),
                namespace: Some(String::from("kube-system")),
                annotations: Some(BTreeMap::from([
                    (String::from("storageclass.kubernetes.io/is-default-class"), String::from("true"))
                ])),
                ..Default::default()
            },
            parameters: Some(BTreeMap::from([
                                (String::from("linodebs.csi.linode.com/encrypted"), String::from("true"))
            ])),
            allow_volume_expansion: Some(true),
            provisioner: String::from("linodebs.csi.linode.com"),
            reclaim_policy: Some(String::from("Retain")),
            volume_binding_mode: Some(String::from("WaitForFirstConsumer")),
            ..Default::default()
        }
    ]
}
