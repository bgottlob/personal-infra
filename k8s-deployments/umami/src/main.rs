use kube_builder::prelude::*;

use std::collections::BTreeMap;
use k8s_openapi::{
    api::{
        apps::v1::Deployment,
        core::v1::{Container, Service, VolumeMount},
    },
    apimachinery::pkg::api::resource::Quantity,
};
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;

const PG_HOST: &str = "main-db-rw.main-db";
const PG_PORT: i32 = 5432;
const DATABASE_NAME: &str = "umami";
const POSTGRES_SECRET: &str = "umami-postgres";
const APP_SECRET: &str = "umami-app";
const GEO_VOLUME: &str = "geo-db";
const GEO_DB_PATH: &str = "/geo/GeoLite2-City.mmdb";
// DB-IP free city database — update month when it gets stale
const GEO_DB_URL: &str = "https://download.db-ip.com/free/dbip-city-lite-2026-06.mmdb.gz";

const NAME: &str = "umami";
const VERSION: &str = "v3.1.0";
const IMAGE: &str = "elestio/umami";
const PORT: i32 = 3000;
const HOSTNAME: &str = "umami.bgottlob.com";

fn labels() -> BTreeMap<String, String> {
    let mut labels: BTreeMap<String, String> = BTreeMap::new();
    labels.insert(String::from("app"), String::from(NAME));
    labels
}

fn create_geo_init_container() -> anyhow::Result<Container> {
    let download_cmd = format!(
        "wget -q -O - {GEO_DB_URL} | gunzip > {GEO_DB_PATH}"
    );

    ContainerBuilder::new()
        .name("geo-downloader")
        .image("busybox:1.37")
        .command(vec!["/bin/sh".into(), "-c".into()])
        .arg(download_cmd)
        .volume_mount(VolumeMount {
            name: GEO_VOLUME.into(),
            mount_path: "/geo".into(),
            ..Default::default()
        })
        .cpu_request(Quantity(String::from("50m")))
        .cpu_limit(Quantity(String::from("200m")))
        .memory_request(Quantity(String::from("64Mi")))
        .memory_limit(Quantity(String::from("64Mi")))
        .build()
}

fn create_deploy() -> anyhow::Result<Deployment> {
    let database_url = format!(
        "postgresql://$(PG_USERNAME):$(PG_PASSWORD)@{}:{}/{}",
        PG_HOST, PG_PORT, DATABASE_NAME
    );

    let container = ContainerBuilder::new()
        .name(NAME)
        .image(format!("{}:{}", IMAGE, VERSION))
        .container_port(PORT, "app", PortProtocol::TCP)
        .env_from_secret("PG_USERNAME", POSTGRES_SECRET, "username")
        .env_from_secret("PG_PASSWORD", POSTGRES_SECRET, "password")
        .env("DATABASE_URL", &database_url)
        .env("DATABASE_TYPE", "postgresql")
        .env_from_secret("APP_SECRET", APP_SECRET, "app_secret")
        .env("GEOLITE_DB_PATH", GEO_DB_PATH)
        .readiness_probe(http_probe(
            "/",
            IntOrString::String(String::from("app")),
            None, None, None, None,
        ))
        .volume_mount(VolumeMount {
            name: GEO_VOLUME.into(),
            mount_path: "/geo".into(),
            ..Default::default()
        })
        .cpu_request(Quantity(String::from("50m")))
        .cpu_limit(Quantity(String::from("500m")))
        .memory_request(Quantity(String::from("512Mi")))
        .memory_limit(Quantity(String::from("1Gi")))
        .build()?;

    let init_container = create_geo_init_container()?;

    DeploymentBuilder::new()
        .name(NAME)
        .replicas(1)
        .selector_match_labels(labels())
        .pod_labels(labels())
        .init_container(init_container)
        .container(container)
        .volume_from_empty_dir(GEO_VOLUME)
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
