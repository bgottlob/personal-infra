use kube_builder::prelude::*;

use std::collections::BTreeMap;

use k8s_openapi::{api::{
    apps::v1::Deployment,
    core::v1::{PersistentVolumeClaim, Service, VolumeMount},
}, apimachinery::pkg::api::resource::Quantity};
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;

const NAME: &str = "kavita";
const VERSION: &str = "0.9.0.2";
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
    let probe = http_probe("/api/health", IntOrString::String(String::from("app")), None, None, None, None);

    let container = ContainerBuilder::new()
        .name(NAME)
        .image(format!("{}:{}", IMAGE, VERSION))
        .container_port(PORT, "app", PortProtocol::TCP)
        .env("TZ", "America/New_York")
        .readiness_probe(probe)
        .cpu_request(Quantity(String::from("150m")))
        .cpu_limit(Quantity(String::from("1000m")))
        .memory_request(Quantity(String::from("300Mi")))
        .memory_limit(Quantity(String::from("1Gi")))
        .volume_mount(
            VolumeMount {
                name: String::from("library"),
                mount_path: String::from("/kavita/library"),
                read_only: Some(false),
                ..Default::default()
            },
        )
        .volume_mount(
            VolumeMount {
                name: String::from("config"),
                mount_path: String::from("/kavita/config"),
                read_only: Some(false),
                ..Default::default()
            },
        )
        .build()?;

    DeploymentBuilder::new()
        .name(NAME)
        .replicas(1)
        .recreate_strategy()
        .selector_match_labels(labels())
        .pod_labels(labels())
        .pod_annotation("backup.velero.io/backup-volumes-excludes", "library")
        .container(container)
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

fn create_redirect_route() -> anyhow::Result<HTTPRoute> {
    HTTPRouteBuilder::new()
        .name(format!("{}-http-redirect", NAME))
        .gateway_parent_ref("envoy-gateway-system", "envoy-public")
        .hostname(HOSTNAME)
        .https_redirect_rule()
        .build()
}

fn create_route() -> anyhow::Result<HTTPRoute> {
    HTTPRouteBuilder::new()
        .name(NAME)
        .listener_set_parent_ref(NAME, format!("{}-https", NAME))
        .service_port_backend_rule(NAME, 80)
        .hostname(HOSTNAME)
        .build()
}

fn create_listener_set() -> anyhow::Result<ListenerSet> {
    ListenerSetBuilder::new()
        .name(NAME)
        .hostname(HOSTNAME)
        .build()
}

fn main() -> anyhow::Result<()> {
    let deploy = create_deploy()?;
    let service = create_service()?;
    let redirect_route = create_redirect_route()?;
    let route = create_route()?;
    let listener_set = create_listener_set()?;
    let library_pvc = create_library_pvc()?;
    let config_pvc = create_config_pvc()?;

    let resources = vec![
        serde_json::value::to_value(deploy)?,
        serde_json::value::to_value(service)?,
        serde_json::value::to_value(redirect_route)?,
        serde_json::value::to_value(route)?,
        serde_json::value::to_value(listener_set)?,
        serde_json::value::to_value(library_pvc)?,
        serde_json::value::to_value(config_pvc)?,
    ];
    println!("{}", serde_json::to_string(&resources).unwrap());
    Ok(())
}
