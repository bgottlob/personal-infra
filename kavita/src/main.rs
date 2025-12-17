use kube_builder::prelude::*;

use std::collections::BTreeMap;

use k8s_openapi::{api::{
    apps::v1::Deployment,
    core::v1::{PersistentVolumeClaim, ResourceRequirements, Service, VolumeMount},
    networking::v1::Ingress,
}, apimachinery::pkg::api::resource::Quantity};
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;

const NAME: &str = "kavita";
const VERSION: &str = "0.8.8";
const IMAGE: &str = "jvmilazz0/kavita";
const PORT: i32 = 5000;
const HOSTNAME: &str = "library.bgottlob.com";
const LIBRARY_PVC_NAME: &str = "kavita-library";
const LIBRARY_STORAGE_REQUEST: &str = "20Gi";
const CONFIG_PVC_NAME: &str = "kavita-config";
const CONFIG_STORAGE_REQUEST: &str = "10Gi";

fn labels() -> BTreeMap<String, String> {
    let mut labels: BTreeMap<String, String> = BTreeMap::new();
    labels.insert(String::from("app"), String::from(NAME));
    labels
}

fn create_deploy() -> anyhow::Result<Deployment> {
    let env = EnvBuilder::new()
        .env("TZ", "America/New_York")
        .build();

    DeploymentBuilder::new()
        .name(NAME)
        .replicas(1)
        .selector_match_labels(labels())
        .pod_labels(labels())
        .container(
            NAME,
            format!("{}:{}", IMAGE, VERSION),
            "app",
            PORT,
            PortProtocol::TCP,
            env,
            Some(http_probe("/api/health", IntOrString::String(String::from("app")))),
            Some(ResourceRequirements {
                requests: Some(BTreeMap::from([
                    (String::from("cpu"), Quantity(String::from("50m"))),
                    (String::from("memory"), Quantity(String::from("128Mi")))
                ])),
                ..Default::default()
            }),
            Some(vec![
                VolumeMount {
                    name: String::from("library"),
                    mount_path: String::from("/kavita/library"),
                    read_only: Some(false),
                    ..Default::default()
                },
                VolumeMount {
                    name: String::from("config"),
                    mount_path: String::from("/kavita/config"),
                    read_only: Some(false),
                    ..Default::default()
                },
            ]),
            None,
        )
        .volume_from_pvc("library", LIBRARY_PVC_NAME)
        .volume_from_pvc("config", CONFIG_PVC_NAME)
        .build()
}

fn create_library_pvc() -> anyhow::Result<PersistentVolumeClaim> {
    PersistentVolumeClaimBuilder::new()
        .name(LIBRARY_PVC_NAME)
        .storage_class_name("linode-block-storage-retain-encrypted")
        .storage_requests(Quantity(String::from(LIBRARY_STORAGE_REQUEST)))
        .build()
}

fn create_config_pvc() -> anyhow::Result<PersistentVolumeClaim> {
    PersistentVolumeClaimBuilder::new()
        .name(CONFIG_PVC_NAME)
        .storage_class_name("linode-block-storage-retain-encrypted")
        .storage_requests(Quantity(String::from(CONFIG_STORAGE_REQUEST)))
        .build()
}

fn create_service() -> anyhow::Result<Service> {
    ServiceBuilder::new()
        .selector(labels())
        .name(NAME)
        .port("app", PortProtocol::TCP, 80, PORT)
        .build()
}

fn create_ingress() -> anyhow::Result<Ingress> {
    IngressBuilder::new()
        .name(NAME)
        .annotation("cert-manager.io/cluster-issuer", "letsencrypt-prod")
        .ingress_class_name("nginx")
        .tls_host(HOSTNAME, NAME)
        .rule(HOSTNAME, "/", "Prefix", NAME, PORT)
        .build()
}

fn main() -> anyhow::Result<()> {
    let deploy = create_deploy()?;
    let service = create_service()?;
    let ingress = create_ingress()?;
    let library_pvc = create_library_pvc()?;
    let config_pvc = create_config_pvc()?;

    let resources = vec![
        serde_json::value::to_value(deploy)?,
        serde_json::value::to_value(service)?,
        serde_json::value::to_value(ingress)?,
        serde_json::value::to_value(library_pvc)?,
        serde_json::value::to_value(config_pvc)?,
    ];
    println!("{}", serde_json::to_string(&resources).unwrap());
    Ok(())
}
