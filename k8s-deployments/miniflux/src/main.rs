use k8s_gateway_api::prelude::HTTPRoute;
use kube_builder::prelude::*;
use sealed_secrets_crd::sealed_secrets::SealedSecret;
use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;
use std::io::Read;

use k8s_openapi::{api::{
    apps::v1::Deployment,
    core::v1::Service,
}, apimachinery::pkg::api::resource::Quantity};

const PG_HOST: &str = "main-db-rw.main-db";
const PG_PORT: i32 = 5432;
const DATABASE_NAME: &str = "miniflux";

const NAME: &str = "miniflux";
const VERSION: &str = "2.0.41";
const IMAGE: &str = "miniflux/miniflux";
const PORT: i32 = 8080;
const HOSTNAME: &str = "miniflux.bgottlob.com";

const DATABASE_SECRET: &str = "miniflux-postgres";
const ADMIN_SECRET: &str = "miniflux-admin";

#[derive(Deserialize, Serialize)]
struct Input {
    #[serde(rename = "sealed-secrets")]
    sealed_secrets: Option<Vec<SealedSecret>>,
}

fn labels() -> BTreeMap<String, String> {
    let mut labels: BTreeMap<String, String> = BTreeMap::new();
    labels.insert(String::from("app"), String::from(NAME));
    labels
}

fn create_deploy() -> anyhow::Result<Deployment> {
    let database_url = format!(
        "postgres://$(PG_USERNAME):$(PG_PASSWORD)@{}:{}/{}?sslmode=disable",
        PG_HOST, PG_PORT, DATABASE_NAME
    );

    let container = ContainerBuilder::new()
        .name(NAME)
        .image(format!("{}:{}", IMAGE, VERSION))
        .env("RUN_MIGRATIONS", "1")
        .env("CREATE_ADMIN", "1")
        .env_from_secret("PG_USERNAME", DATABASE_SECRET, "username")
        .env_from_secret("PG_PASSWORD", DATABASE_SECRET, "password")
        .env("DATABASE_URL", &database_url)
        .env_from_secret("ADMIN_USERNAME", ADMIN_SECRET, "username")
        .env_from_secret("ADMIN_PASSWORD", ADMIN_SECRET, "password")
        .container_port(PORT, "app", PortProtocol::TCP)
        .cpu_request(Quantity(String::from("50m")))
        .cpu_limit(Quantity(String::from("500m")))
        .memory_request(Quantity(String::from("128Mi")))
        .memory_limit(Quantity(String::from("128Mi")))
        .build()?;

    DeploymentBuilder::new()
        .name(NAME)
        .replicas(1)
        .selector_match_labels(labels())
        .pod_labels(labels())
        .container(container)
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
        .gateway_parent_ref("envoy-gateway-system", "envoy-public")
        .hostname(HOSTNAME)
        .service_port_backend_rule(NAME, 80)
        .build()
}

fn read_sealed_secrets_from_stdin() -> anyhow::Result<Vec<serde_json::Value>> {
    let mut stdin = String::new();
    std::io::stdin().read_to_string(&mut stdin)?;

    if stdin.trim().is_empty() {
        return Ok(vec![]);
    }

    let input: Input = serde_json::from_str(&stdin)?;
    input.sealed_secrets
        .unwrap_or_default()
        .into_iter()
        .map(|s| Ok(serde_json::to_value(s)?))
        .collect()
}

fn main() -> anyhow::Result<()> {
    let deploy = create_deploy()?;
    let service = create_service()?;
    let route = create_route()?;
    let sealed_secrets = read_sealed_secrets_from_stdin()?;

    let mut resources: Vec<Vec<serde_json::Value>> = Vec::new();
    resources.push(sealed_secrets);
    resources.push(vec![
        serde_json::value::to_value(deploy)?,
        serde_json::value::to_value(service)?,
        serde_json::value::to_value(route)?,
    ]);

    println!("{}", serde_json::to_string(&resources).unwrap());
    Ok(())
}
