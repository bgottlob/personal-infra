use k8s_gateway_api::prelude::*;
use kube_builder::prelude::*;

use std::collections::BTreeMap;

use k8s_openapi::{api::{
    apps::v1::Deployment,
    core::v1::{Secret, Service},
    networking::v1::Ingress,
}, apimachinery::pkg::api::resource::Quantity};

const PG_HOST: &str = "main-db-rw.main-db";
const PG_PORT: i32 = 5432;
const DATABASE_NAME: &str = "wallabag";

const NAME: &str = "wallabag";
const VERSION: &str = "2.6.14";
const IMAGE: &str = "wallabag/wallabag";
const PORT: i32 = 80;
const HOSTNAME: &str = "wallabag.bgottlob.com";

const DATABASE_SECRET: &str = "wallabag-postgres";

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
        .cpu_request(Quantity(String::from("50m")))
        .memory_request(Quantity(String::from("256Mi")))
	.env_from_secret("SYMFONY__ENV__DATABASE_HOST", DATABASE_SECRET, "host")
	.env_from_secret("SYMFONY__ENV__DATABASE_PORT", DATABASE_SECRET, "port")
        .env("SYMFONY__ENV__DATABASE_DRIVER", "pdo_pgsql")
        .env("SYMFONY__ENV__DATABASE_NAME", DATABASE_NAME)
        .env("SYMFONY__ENV__DOMAIN_NAME", format!("https://{}", HOSTNAME))
        .env("SYMFONY__ENV__FOSUSER_REGISTRATION", "false")
        .env("SYMFONY__ENV__SERVER_NAME", "Brandon's Wallabag")
        .env_from_secret("POSTGRES_PASSWORD", DATABASE_SECRET, "password")
        .env_from_secret("POSTGRES_USER", DATABASE_SECRET, "user")
        .env_from_secret("SYMFONY__ENV__DATABASE_PASSWORD", DATABASE_SECRET, "password")
        .env_from_secret("SYMFONY__ENV__DATABASE_USER", DATABASE_SECRET, "user")
        .build()?;

    DeploymentBuilder::new()
        .name(NAME)
        .replicas(1)
        .selector_match_labels(labels())
        .pod_labels(labels())
        .container(container)
        .build()
}

fn create_secret() -> anyhow::Result<Secret> {
    SecretBuilder::new()
        .name(DATABASE_SECRET)
        .value("host", PG_HOST)
        .value("password", env!("POSTGRES_PASSWORD"))
        .value("port", format!("{}", PG_PORT).as_str())
        .value("user", env!("POSTGRES_USERNAME"))
        .build()
}

fn create_service() -> anyhow::Result<Service> {
    ServiceBuilder::new()
        .selector(labels())
        .name(NAME)
        .port("app", PortProtocol::TCP, 80, PORT)
        .build()
}

fn create_http_route() -> anyhow::Result<HTTPRoute> {
    HTTPRouteBuilder::new()
        .name(NAME)
        .gateway_parent_ref("envoy-gateway-system", "envoy-public")
        .hostname("wallabag.bgottlob.com")
        .service_port_backend_rule(NAME, PORT)
        .build()
}

fn main() -> anyhow::Result<()> {
    let deploy = create_deploy()?;
    let service = create_service()?;
    let secret = create_secret()?;

    let route = create_http_route()?;

    let resources = vec![
        serde_json::value::to_value(deploy)?,
        serde_json::value::to_value(service)?,
        serde_json::value::to_value(secret)?,
        serde_json::value::to_value(route)?,
    ];
    println!("{}", serde_json::to_string(&resources).unwrap());
    Ok(())
}
