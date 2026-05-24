use std::collections::BTreeMap;

use k8s_openapi::{api::{apps::v1::Deployment, core::v1::Service}, apimachinery::pkg::api::resource::Quantity};
use kube_builder::prelude::*;

const NAME: &str = "blog";
const IMAGE_TAG: &str = "latest";

fn labels() -> BTreeMap<String, String> {
    let mut labels: BTreeMap<String, String> = BTreeMap::new();
    labels.insert("app".into(), NAME.into());
    labels
}

fn create_deployment() -> anyhow::Result<Deployment> {
    let image = format!("registry.bgottlob.com/{}:{}", NAME, IMAGE_TAG);
    let memory = Quantity(String::from("8Mi"));
    let container = ContainerBuilder::new()
        .name(NAME)
        .image(image)
        .container_port(80, "app", PortProtocol::TCP)
        .cpu_request(Quantity(String::from("5m")))
        .cpu_limit(Quantity(String::from("100m")))
        .memory_request(memory.clone())
        .memory_limit(memory)
        .build()?;

    DeploymentBuilder::new()
        .replicas(1)
        .name(NAME)
        .container(container)
        .selector_match_labels(labels())
        .use_private_registry()
        .build()
}

fn create_service() -> anyhow::Result<Service> {
    ServiceBuilder::new()
        .name(NAME)
        .selector(labels())
        .port("http", PortProtocol::TCP, 80, 80)
        .build()
}

fn create_redirect_route() -> anyhow::Result<HTTPRoute> {
    HTTPRouteBuilder::new()
        .name(format!("{}-http-redirect", NAME))
        .gateway_parent_ref("envoy-gateway-system", "envoy-public")
        .hostname("bgottlob.com")
        .https_redirect_rule()
        .build()
}

fn create_route() -> anyhow::Result<HTTPRoute> {
    HTTPRouteBuilder::new()
        .name(NAME)
        .listener_set_parent_ref(NAME, format!("{}-https", NAME))
        .service_port_backend_rule(NAME, 80)
        .hostname("bgottlob.com")
        .build()
}

fn create_listener_set() -> anyhow::Result<ListenerSet> {
    ListenerSetBuilder::new()
        .name(NAME)
        .hostname("bgottlob.com")
        .build()
}

fn main() -> anyhow::Result<()> {
    let sealed_secrets = read_sealed_secrets_from_stdin()?;
    let deploy = create_deployment()?;
    let service = create_service()?;
    let redirect_route = create_redirect_route()?;
    let route = create_route()?;
    let listener_set = create_listener_set()?;

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
    ]);
    println!("{}", serde_json::to_string(&resources)?);
    Ok(())
}
