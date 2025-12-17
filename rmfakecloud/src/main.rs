use kube_builder::prelude::*;

use std::collections::BTreeMap;

use k8s_openapi::{api::{
    apps::v1::Deployment,
    core::v1::{PersistentVolumeClaim, ResourceRequirements, Service, VolumeMount},
    networking::v1::Ingress,
}, apimachinery::pkg::api::resource::Quantity};

const NAME: &str = "rmfakecloud";
const VERSION: &str = "0.0.27";
const IMAGE: &str = "ddvk/rmfakecloud";
const PORT: i32 = 3000;
const HOSTNAME: &str = "remarkable.bgottlob.com";
const PVC_NAME: &str = "rmfakecloud-data";
const STORAGE_REQUEST: &str = "10Gi";

fn labels() -> BTreeMap<String, String> {
    let mut labels: BTreeMap<String, String> = BTreeMap::new();
    labels.insert(String::from("app"), String::from(NAME));
    labels
}

fn create_deploy() -> anyhow::Result<Deployment> {
    let volume_mount_path = "/data/rmfakecloud";
    let env = EnvBuilder::new()
        .env("DATADIR", volume_mount_path)
        .env("STORAGE_URL", format!("https://{}", HOSTNAME))
        .env("PORT", format!("{}", PORT))
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
            None,
            Some(ResourceRequirements {
                requests: Some(BTreeMap::from([
                    (String::from("cpu"), Quantity(String::from("50m"))),
                    (String::from("memory"), Quantity(String::from("128Mi")))
                ])),
                ..Default::default()
            }),
            Some(vec![VolumeMount {
                name: String::from("data"),
                mount_path: volume_mount_path.to_string(),
                read_only: Some(false),
                ..Default::default()
            }]),
            None,
        )
        .volume_from_pvc("data", PVC_NAME)
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
        .annotation("nginx.ingress.kubernetes.io/proxy-body-size", "500m")
        .ingress_class_name("nginx")
        .tls_host(HOSTNAME, NAME)
        .rule(HOSTNAME, "/", "Prefix", NAME, 80)
        .build()
}

fn create_pvc() -> anyhow::Result<PersistentVolumeClaim> {
    PersistentVolumeClaimBuilder::new()
        .storage_class_name("linode-block-storage-retain-encrypted")
        .name(PVC_NAME)
        .storage_requests(Quantity(String::from(STORAGE_REQUEST)))
        .build()
}

fn main() -> anyhow::Result<()> {
    let deploy = create_deploy()?;
    let service = create_service()?;
    let ingress = create_ingress()?;
    let pvc = create_pvc()?;

    let resources = vec![
        serde_json::value::to_value(deploy)?,
        serde_json::value::to_value(service)?,
        serde_json::value::to_value(ingress)?,
        serde_json::value::to_value(pvc)?,
    ];
    println!("{}", serde_json::to_string(&resources).unwrap());
    Ok(())
}
