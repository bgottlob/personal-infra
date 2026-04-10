use kube_builder::prelude::*;
use std::collections::BTreeMap;

use k8s_openapi::{
    api::{apps::v1::Deployment, core::v1::{PersistentVolumeClaim, ResourceRequirements, Secret}},
    apimachinery::pkg::{api::resource::Quantity, util::intstr::IntOrString},
};

const NAME: &str = "vikunja";
const VERSION: &str = "0.24.6";

const PG_HOST: &str = "main-db-rw.main-db";
const DATABASE_NAME: &str = "vikunja";
const DB_SECRET_NAME: &str = "vikunja-database";

const DATA_PVC_NAME: &str = "vikunja-data";

fn create_db_secret() -> anyhow::Result<Secret> {
    SecretBuilder::new()
        .name(DB_SECRET_NAME)
        .value("HOST", PG_HOST)
        .value("DATABASE", DATABASE_NAME)
        .value("USERNAME", env!("POSTGRES_USERNAME"))
        .value("PASSWORD", env!("POSTGRES_PASSWORD"))
        .build()
}

fn create_deploy() -> anyhow::Result<Deployment> {
    let mut labels: BTreeMap<String, String> = BTreeMap::new();
    labels.insert(String::from("app"), String::from(NAME));

    let env = EnvBuilder::new()
        .env_from_secret("VIKUNJA_DATABASE_DATABASE", DB_SECRET_NAME, "DATABASE")
        .env_from_secret("VIKUNJA_DATABASE_HOST", DB_SECRET_NAME, "HOST")
        .env_from_secret("VIKUNJA_DATABASE_NAME", DB_SECRET_NAME, "DATABASE")
        .env_from_secret("VIKUNJA_DATABASE_USER", DB_SECRET_NAME, "USERNAME")
        .env_from_secret("VIKUNJA_DATABASE_PASSWORD", DB_SECRET_NAME, "PASSWORD")
        .env("VIKUNJA_DATABASE_TYPE", "postgres")
        .build();

    DeploymentBuilder::new()
        .name(NAME)
        .replicas(1)
        .selector_match_labels(labels.clone())
        .pod_labels(labels.clone())
        .container(
            "vikunja",
            format!("vikunja/vikunja:{}", VERSION),
            "http",
            3456,
            PortProtocol::TCP,
            env,
            Some(http_probe("/api/v1/info", IntOrString::String("http".into()))),
            Some(
                ResourceRequirements {
                    requests: Some(BTreeMap::from([
                        (String::from("cpu"), Quantity(String::from("50m"))),
                        (String::from("memory"), Quantity(String::from("128Mi"))),
                    ])),
                    ..Default::default()
                }
            ),
            None,
        )
        .tailscale_container()
        .volume_from_pvc("datb", DATA_PVC_NAME)
        // TODO - get from tailscale module
        .service_account_name("tailscale")
        .build()
}

fn create_pvc() -> anyhow::Result<PersistentVolumeClaim> {
    PersistentVolumeClaimBuilder::new()
        .name(DATA_PVC_NAME)
        .storage_class_name("linode-block-storage-retain-encrypted")
        .storage_requests(Quantity("10Gi".to_string()))
        .build()
}

fn main() -> anyhow::Result<()> {
    let deploy = create_deploy()?;
    let secret = create_db_secret()?;
    let pvc = create_pvc()?;

    let ts_secret = tailscale::secret(env!("TS_AUTHKEY"));
    let (ts_role, ts_role_binding, ts_service_account) = tailscale::rbac();

    let resources = vec![
        serde_json::value::to_value(deploy)?,
        serde_json::value::to_value(secret)?,
        serde_json::value::to_value(ts_secret)?,
        serde_json::value::to_value(ts_role)?,
        serde_json::value::to_value(ts_role_binding)?,
        serde_json::value::to_value(ts_service_account)?,
        serde_json::value::to_value(pvc)?,
    ];
    println!("{}", serde_json::to_string(&resources).unwrap());
    Ok(())
}
