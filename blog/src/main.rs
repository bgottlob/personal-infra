use std::collections::BTreeMap;

use k8s_openapi::api::{apps::v1::Deployment, core::v1::Service, networking::v1::Ingress};
use kube_builder::prelude::*;

const NAME: &str = "blog";
const IMAGE_TAG: &str = "latest";

fn labels() -> BTreeMap<String, String> {
    let mut labels: BTreeMap<String, String> = BTreeMap::new();
    labels.insert("app".into(), NAME.into());
    labels
}

fn create_deployment() -> anyhow::Result<Deployment> {
    DeploymentBuilder::new()
        .replicas(1)
        .name(NAME)
        .selector_match_labels(labels())
        .container(
            NAME,
            format!("{}/{}:{}", env!("REGISTRY_SERVER"), NAME, IMAGE_TAG),
            "app",
            80,
            PortProtocol::TCP,
            Vec::new(),
            None
        )
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

fn create_ingress() -> anyhow::Result<Ingress> {
    IngressBuilder::new()
        .name(NAME)
        .ingress_class_name("nginx")
        .rule("bgottlob.com", "/", "Prefix", NAME, 80)
        .tls_host("bgottlob.com", NAME)
        .build()
}

fn main() -> anyhow::Result<()> {
    let deploy = create_deployment()?;
    let service = create_service()?;
    let ingress = create_ingress()?;
    let docker_secret = docker_registry_secret(
        env!("REGISTRY_SERVER").into(),
        env!("REGISTRY_USERNAME").into(),
        env!("REGISTRY_PASSWORD").into(),
        env!("REGISTRY_EMAIL").into(),
    );

    let resources = vec![
        serde_json::value::to_value(docker_secret)?,
        serde_json::value::to_value(deploy)?,
        serde_json::value::to_value(service)?,
        serde_json::value::to_value(ingress)?,
    ];
    println!("{}", serde_json::to_string(&resources)?);
    Ok(())
}
