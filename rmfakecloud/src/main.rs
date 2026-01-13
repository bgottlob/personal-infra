use k8s_gateway_api::prelude::HTTPRoute;
use kube_builder::prelude::*;

use std::collections::BTreeMap;

use k8s_openapi::{api::{
    apps::v1::Deployment,
    core::v1::{PersistentVolumeClaim, Service, VolumeMount},
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

    let container = ContainerBuilder::new()
        .name(NAME)
        .image(format!("{}:{}", IMAGE, VERSION))
        .env("DATADIR", volume_mount_path)
        .env("STORAGE_URL", format!("https://{}", HOSTNAME))
        .env("PORT", format!("{}", PORT))
        .container_port(PORT, "app", PortProtocol::TCP)
        .cpu_request(Quantity(String::from("50m")))
        .memory_request(Quantity(String::from("128Mi")))
        .volume_mount(
            VolumeMount {
                name: String::from("data"),
                mount_path: volume_mount_path.to_string(),
                read_only: Some(false),
                ..Default::default()
            }
        )
        .build()?;

    DeploymentBuilder::new()
        .name(NAME)
        .replicas(1)
        .selector_match_labels(labels())
        .pod_labels(labels())
        .container(container)
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

fn create_route() -> anyhow::Result<HTTPRoute> {
    HTTPRouteBuilder::new()
        .name(NAME)
        .service_port_backend_rule(NAME, 80)
        .gateway_parent_ref("envoy-gateway-system", "envoy-public")
        .hostname(HOSTNAME)
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
    let route = create_route()?;
    let pvc = create_pvc()?;

    let resources = vec![
        serde_json::value::to_value(deploy)?,
        serde_json::value::to_value(service)?,
        serde_json::value::to_value(route)?,
        serde_json::value::to_value(pvc)?,
    ];
    println!("{}", serde_json::to_string(&resources).unwrap());
    Ok(())
}
