use kube_builder::prelude::*;

use std::collections::BTreeMap;

use k8s_openapi::{api::{
    apps::v1::Deployment,
    core::v1::{ResourceRequirements, Secret, Service},
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
    let env = EnvBuilder::new()
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
    let ingress = create_ingress()?;

    let resources = vec![
        serde_json::value::to_value(deploy)?,
        serde_json::value::to_value(service)?,
        serde_json::value::to_value(secret)?,
        serde_json::value::to_value(ingress)?,
    ];
    println!("{}", serde_json::to_string(&resources).unwrap());
    Ok(())
}
