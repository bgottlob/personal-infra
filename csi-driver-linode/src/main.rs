use std::{collections::BTreeMap, env};
use k8s_openapi::api::storage::v1::StorageClass;
use kube::core::ObjectMeta;
use serde::Deserialize;
use serde_norway::Value;

fn main() -> anyhow::Result<()> {
    let helm_out_str = include_str!(concat!(env!("OUT_DIR"), "/helm-output.yaml"));
    let mut resources: Vec<serde_json::Value> = Vec::new();
    for document in serde_norway::Deserializer::from_str(helm_out_str) {
        let value = Value::deserialize(document)?;
        if value != Value::Null {
            // Remove the default storage class annotation from any storage
            // classes that are set as default by the Helm chart; the encrypted
            // storage class will be the only default
            // TODO this is janky but it works
            if *value.get("kind").unwrap() == Value::String(String::from("StorageClass")) {
                let mut sc: StorageClass = serde_norway::from_value(value).unwrap();
                if sc.metadata.annotations.is_some() {
                    sc.metadata.annotations.as_mut().unwrap().remove("storageclass.kubernetes.io/is-default-class");
                }
                resources.push(serde_json::to_value(sc)?);
            } else {
                resources.push(serde_json::to_value(value)?);
            }
        }
    }

    for sc in create_encrypted_storage_classes() {
        let sc_value = serde_json::to_value(sc)?;
        resources.push(sc_value);
    }

    println!("{}", serde_json::to_string(&resources).unwrap());
    Ok(())
}

fn create_encrypted_storage_classes() -> Vec<StorageClass> {
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
            ..Default::default()
        }
    ]
}
