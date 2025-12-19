use kube_builder::prelude::*;

use std::collections::BTreeMap;

use k8s_openapi::{api::{
    apps::v1::Deployment,
    core::v1::{ConfigMap, Secret, Service, VolumeMount},
    networking::v1::Ingress,
}, apimachinery::pkg::{apis::meta::v1::ObjectMeta}};

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
    let env = EnvBuilder::new()
        .env_from_secret("REGISTRY_STORAGE_S3_ACCESSKEY", S3_SECRET_NAME, "accesskey")
        .env_from_secret("REGISTRY_STORAGE_S3_SECRETKEY", S3_SECRET_NAME, "secretkey")
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
            None,
            Some(vec![
                VolumeMount {
                    name: String::from("registry-config"),
                    mount_path: String::from("/etc/docker/registry"),
                    read_only: Some(true),
                    ..Default::default()
                },
                VolumeMount {
                    name: String::from("registry-htpasswd-secret"),
                    mount_path: String::from("/auth"),
                    read_only: Some(true),
                    ..Default::default()
                }
            ]),
            None,
        )
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

fn create_s3_secret() -> anyhow::Result<Secret> {
    SecretBuilder::new()
        .name(S3_SECRET_NAME)
        .value("accesskey", env!("S3_ACCESS_KEY"))
        .value("secretkey", env!("S3_SECRET_KEY"))
        .build()
}

fn create_auth_secret() -> anyhow::Result<Secret> {
    SecretBuilder::new()
        .name(S3_SECRET_NAME)
        .value("htpasswd", env!("AUTH_HTPASSWD"))
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
        // Allows images of any size to be uploaded
        .annotation("nginx.ingress.kubernetes.io/proxy-body-size", "0")
        .annotation("nginx.ingress.kubernetes.io/proxy-read-timeout", "6000")
        .annotation("nginx.ingress.kubernetes.io/proxy-send-timeout", "6000")
        .ingress_class_name("nginx")
        .tls_host(HOSTNAME, NAME)
        .rule(HOSTNAME, "/", "Prefix", NAME, 80)
        .build()
}

fn main() -> anyhow::Result<()> {
    let deploy = create_deploy()?;
    let service = create_service()?;
    let ingress = create_ingress()?;
    let s3_secret = create_s3_secret()?;
    let auth_secret = create_auth_secret()?;
    let configmap = create_configmap();

    let resources = vec![
        serde_json::value::to_value(deploy)?,
        serde_json::value::to_value(service)?,
        serde_json::value::to_value(ingress)?,
        serde_json::value::to_value(s3_secret)?,
        serde_json::value::to_value(auth_secret)?,
        serde_json::value::to_value(configmap)?,
    ];
    println!("{}", serde_json::to_string(&resources).unwrap());
    Ok(())
}
