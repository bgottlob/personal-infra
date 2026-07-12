use std::collections::BTreeMap;
use k8s_openapi::api::storage::v1::StorageClass;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

fn main() -> anyhow::Result<()> {
    let storage_classes: Vec<serde_json::Value> = create_encrypted_storage_classes()
        .into_iter()
        .map(|sc| serde_json::to_value(sc))
        .collect::<Result<_, _>>()?;
    println!("{}", serde_json::to_string(&vec![storage_classes])?);
    Ok(())
}

fn create_encrypted_storage_classes() -> Vec<StorageClass> {
    // WaitForFirstConsumer delays volume provisioning until a pod is scheduled, ensuring
    // the volume is created in the same zone as the pod and avoiding topology mismatches.
    vec![
        StorageClass {
            metadata: ObjectMeta {
                name: Some(String::from("linode-block-storage-encrypted")),
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
        },
    ]
}
