use anyhow::anyhow;
use std::collections::BTreeMap;
use k8s_openapi::{api::core::v1::{PersistentVolumeClaim, PersistentVolumeClaimSpec, VolumeResourceRequirements}, apimachinery::pkg::{api::resource::Quantity, apis::meta::v1::ObjectMeta}};

#[derive(Default)]
pub struct PersistentVolumeClaimBuilder {
    name: Option<String>,
    storage_requests: Option<Quantity>,
    storage_class_name: Option<String>,
    volume_name: Option<String>,
}

impl PersistentVolumeClaimBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    //pub fn access_mode() 

    pub fn name<S: Into<String>>(&mut self, name: S) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    pub fn storage_requests(&mut self, sr: Quantity) -> &mut Self {
        self.storage_requests = Some(sr.into());
        self
    }

    pub fn storage_class_name<S: Into<String>>(&mut self, sc: S) -> &mut Self {
        self.storage_class_name = Some(sc.into());
        self
    }

    pub fn volume_name<S: Into<String>>(&mut self, volume_name: S) -> &mut Self {
        self.volume_name = Some(volume_name.into());
        self
    }

    pub fn build(&self) -> anyhow::Result<PersistentVolumeClaim> {
        let name = self.name.clone().ok_or(anyhow!("The PVC must have a name"))?;
        let storage_class_name = self.storage_class_name.clone().unwrap_or("linode-block-storage-retain".to_string());
        // TODO verify this string will be accepted by the k8s API
        // use https://docs.rs/kube_quantity/latest/kube_quantity/
        let storage_requests = self.storage_requests
            .clone()
            .unwrap_or(Quantity("10Gi".to_string()));
        let mut requests: BTreeMap<String, Quantity> = BTreeMap::new();
        requests.insert("storage".into(), storage_requests);

        let pvc = PersistentVolumeClaim {
            metadata: ObjectMeta {
                name: Some(name),
                ..Default::default()
            },
            spec: Some(PersistentVolumeClaimSpec {
                // TODO, also access_modes is required
                access_modes: Some(vec!["ReadWriteOnce".into()]),
                storage_class_name: Some(storage_class_name),
                resources: Some(VolumeResourceRequirements {
                    requests: Some(requests),
                    ..Default::default()
                }),
                volume_name: self.volume_name.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        Ok(pvc)
    }
}
