use kube_builder::prelude::*;
use std::collections::BTreeMap;

use k8s_openapi::{
    api::{
        apps::v1::Deployment,
        core::v1::{PersistentVolumeClaim, Service, VolumeMount},
    },
    apimachinery::pkg::{api::resource::Quantity, util::intstr::IntOrString},
};

const NAME: &str = "vikunja";
const VERSION: &str = "0.24.6";
const PORT: i32 = 3456;

const DB_SECRET_NAME: &str = "vikunja-database";
const DATA_PVC_NAME: &str = "vikunja-data";
const DATA_VOLUME_NAME: &str = "data";
const DATA_MOUNT_PATH: &str = "/app/vikunja/files";

fn labels() -> BTreeMap<String, String> {
    let mut labels = BTreeMap::new();
    labels.insert("app".into(), NAME.into());
    labels
}

fn create_deploy() -> anyhow::Result<Deployment> {
    let container = ContainerBuilder::new()
        .name(NAME)
        .image(format!("vikunja/vikunja:{}", VERSION))
        .container_port(PORT, "http", PortProtocol::TCP)
        .env_from_secret("VIKUNJA_DATABASE_HOST", DB_SECRET_NAME, "HOST")
        .env_from_secret("VIKUNJA_DATABASE_DATABASE", DB_SECRET_NAME, "DATABASE")
        .env_from_secret("VIKUNJA_DATABASE_USER", DB_SECRET_NAME, "USERNAME")
        .env_from_secret("VIKUNJA_DATABASE_PASSWORD", DB_SECRET_NAME, "PASSWORD")
        .env("VIKUNJA_DATABASE_TYPE", "postgres")
        .env("VIKUNJA_FILES_BASEPATH", DATA_MOUNT_PATH)
        .cpu_request(Quantity("50m".into()))
        .memory_request(Quantity("128Mi".into()))
        .readiness_probe(http_probe(
            "/api/v1/info",
            IntOrString::String("http".into()),
            None, None, None, None,
        ))
        .volume_mount(VolumeMount {
            name: DATA_VOLUME_NAME.into(),
            mount_path: DATA_MOUNT_PATH.into(),
            ..Default::default()
        })
        .build()?;

    DeploymentBuilder::new()
        .name(NAME)
        .replicas(1)
        .selector_match_labels(labels())
        .pod_labels(labels())
        .container(container)
        .volume_from_pvc(DATA_VOLUME_NAME, DATA_PVC_NAME)
        .build()
}

fn create_pvc() -> anyhow::Result<PersistentVolumeClaim> {
    PersistentVolumeClaimBuilder::new()
        .name(DATA_PVC_NAME)
        .storage_class_name("linode-block-storage-retain-encrypted")
        .storage_requests(Quantity("10Gi".into()))
        .build()
}

fn create_service() -> anyhow::Result<Service> {
    ServiceBuilder::new()
        .name(NAME)
        .selector(labels())
        .port("http", PortProtocol::TCP, 80, PORT)
        .load_balancer_class("tailscale")
        .annotation("tailscale.com/hostname", NAME)
        .build()
}

fn main() -> anyhow::Result<()> {
    let sealed_secrets = read_sealed_secrets_from_stdin()?;
    let deploy = create_deploy()?;
    let pvc = create_pvc()?;
    let service = create_service()?;

    let mut resources: Vec<Vec<serde_json::Value>> = Vec::new();
    if !sealed_secrets.is_empty() {
        resources.push(sealed_secrets);
    }
    resources.push(vec![
        serde_json::value::to_value(deploy)?,
        serde_json::value::to_value(service)?,
        serde_json::value::to_value(pvc)?,
    ]);
    println!("{}", serde_json::to_string(&resources)?);
    Ok(())
}
