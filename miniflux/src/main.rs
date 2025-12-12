use kube_builder::prelude::*;

use std::collections::BTreeMap;

use k8s_openapi::api::{
    apps::v1::Deployment,
    core::v1::{Secret, Service},
    networking::v1::Ingress,
};

const PG_HOST: &str = "cnpg-database-cluster-rw.cnpg-database";
const PG_PORT: i32 = 5432;
const DATABASE_NAME: &str = "miniflux";

const NAME: &str = "miniflux";
const VERSION: &str = "2.0.41";
const IMAGE: &str = "miniflux/miniflux";
const PORT: i32 = 8080;
const HOSTNAME: &str = "miniflux.bgottlob.com";

const DATABASE_SECRET: &str = "miniflux-postgres";
const ADMIN_SECRET: &str = "miniflux-admin";

fn labels() -> BTreeMap<String, String> {
    let mut labels: BTreeMap<String, String> = BTreeMap::new();
    labels.insert(String::from("app"), String::from(NAME));
    labels
}

fn create_deploy() -> anyhow::Result<Deployment> {
    let env = EnvBuilder::new()
        .env("RUN_MIGRATIONS", "1")
        .env("CREATE_ADMIN", "1")
        .env_from_secret("DATABASE_URL", DATABASE_SECRET, "database_url")
        .env_from_secret("ADMIN_USERNAME", ADMIN_SECRET, "username")
        .env_from_secret("ADMIN_PASSWORD", ADMIN_SECRET, "password")
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
            })
        )
        .build()
}

fn create_secret() -> anyhow::Result<Secret> {
    let db_url = format!(
        "postgres://{}:{}@{}:{}/{}?sslmode=disable",
        env!("POSTGRES_USERNAME"),
        env!("POSTGRES_PASSWORD"),
        PG_HOST,
        PG_PORT,
        DATABASE_NAME
    );

    SecretBuilder::new()
        .name(DATABASE_SECRET)
        .value("database_url", &db_url)
        .build()
}

fn create_admin_secret() -> anyhow::Result<Secret> {
    SecretBuilder::new()
        .name(ADMIN_SECRET)
        .value("username", env!("MINIFLUX_ADMIN_USERNAME"))
        .value("password", env!("MINIFLUX_ADMIN_PASSWORD"))
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
    let secret = create_secret()?;
    let admin_secret = create_admin_secret()?;
    let ingress = create_ingress()?;

    let resources = vec![
        serde_json::value::to_value(deploy)?,
        serde_json::value::to_value(service)?,
        serde_json::value::to_value(secret)?,
        serde_json::value::to_value(admin_secret)?,
        serde_json::value::to_value(ingress)?,
    ];
    println!("{}", serde_json::to_string(&resources).unwrap());
    Ok(())
}
