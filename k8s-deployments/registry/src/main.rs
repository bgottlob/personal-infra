use kube_builder::prelude::*;

use std::collections::BTreeMap;

use k8s_openapi::{api::{
    apps::v1::Deployment,
    core::v1::{ConfigMap, Service, VolumeMount},
}, apimachinery::pkg::{api::resource::Quantity, apis::meta::v1::ObjectMeta}};

const NAME: &str = "registry";
const VERSION: &str = "2.8.3";
const IMAGE: &str = "registry";
const PORT: i32 = 5000;
const HOSTNAME: &str = "registry.bgottlob.com";

const AUTH_SECRET_NAME: &str = "registry-htpasswd-secret";
const S3_SECRET_NAME: &str = "registry-s3-secret";
const CONFIGMAP_NAME: &str = "registry-config";

fn labels() -> BTreeMap<String, String> {
    let mut labels: BTreeMap<String, String> = BTreeMap::new();
    labels.insert(String::from("app"), String::from(NAME));
    labels
}

fn create_deploy() -> anyhow::Result<Deployment> {
    let container = ContainerBuilder::new()
        .name(NAME)
        .image(format!("{}:{}", IMAGE, VERSION))
        .container_port(PORT, "app", PortProtocol::TCP)
        .env_from_secret("REGISTRY_STORAGE_S3_ACCESSKEY", S3_SECRET_NAME, "accesskey")
        .env_from_secret("REGISTRY_STORAGE_S3_SECRETKEY", S3_SECRET_NAME, "secretkey")
        .cpu_request(Quantity(String::from("25m")))
        .cpu_limit(Quantity(String::from("200m")))
        .memory_request(Quantity(String::from("64Mi")))
        .memory_limit(Quantity(String::from("64Mi")))
        .volume_mount(
            VolumeMount {
                name: String::from("registry-config"),
                mount_path: String::from("/etc/docker/registry"),
                read_only: Some(true),
                ..Default::default()
            },
        )
        .volume_mount(
            VolumeMount {
                name: String::from("registry-htpasswd-secret"),
                mount_path: String::from("/auth"),
                read_only: Some(true),
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
        .volume_from_configmap("registry-config", CONFIGMAP_NAME, "config.yml", "config.yml")
        .volume_from_secret("registry-htpasswd-secret", AUTH_SECRET_NAME)
        .build()
}

fn create_configmap() -> ConfigMap {
    let cm_data = include_str!("configmap-data.yaml");
    ConfigMap {
        metadata: ObjectMeta {
            name: Some(CONFIGMAP_NAME.to_string()),
            ..Default::default()
        },
        data: Some(BTreeMap::from([
            (String::from("config.yml"), cm_data.to_string())
        ])),
        ..Default::default()
    }
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
        .hostname(HOSTNAME)
        .service_port_backend_rule(NAME, 80)
        .build()
}

fn create_listener_set() -> anyhow::Result<ListenerSet> {
    ListenerSetBuilder::new()
        .name(NAME)
        .hostname(HOSTNAME)
        .build()
}

fn main() -> anyhow::Result<()> {
    let sealed_secrets = read_sealed_secrets_from_stdin()?;
    let deploy = create_deploy()?;
    let service = create_service()?;
    let redirect_route = create_redirect_route()?;
    let route = create_route()?;
    let listener_set = create_listener_set()?;
    let configmap = create_configmap();

    let mut resources: Vec<Vec<serde_json::Value>> = Vec::new();
    if !sealed_secrets.is_empty() {
        resources.push(sealed_secrets);
    }
    resources.push(vec![
        serde_json::value::to_value(deploy)?,
        serde_json::value::to_value(service)?,
        serde_json::value::to_value(redirect_route)?,
        serde_json::value::to_value(route)?,
        serde_json::value::to_value(listener_set)?,
        serde_json::value::to_value(configmap)?,
    ]);
    println!("{}", serde_json::to_string(&resources)?);
    Ok(())
}
