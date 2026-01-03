use kube_builder::prelude::*;

use std::collections::BTreeMap;
use k8s_openapi::{
    api::{
        apps::v1::Deployment,
        core::v1::{
            {% if use_volume %}PersistentVolumeClaim, {% endif %}Secret, Service{% if use_volume %}, VolumeMount{% endif %}
        },
        {% if ingress_hostname != "" %}networking::v1::Ingress,{% endif %}
    },
    apimachinery::pkg::api::resource::Quantity
};
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;

{% if use_postgres %}
const PG_HOST: &str = "main-db-rw.main-db";
const PG_PORT: i32 = 5432;
const DATABASE_NAME: &str = "{{ project-name }}";
const DATABASE_SECRET: &str = "{{ project-name }}-postgres";
{% endif %}
const NAME: &str = "{{ project-name }}";
const VERSION: &str = "{{ version }}";
const IMAGE: &str = "{{ image }}";
const PORT: i32 = 8080;
{% if ingress_hostname != "" %}
const HOSTNAME: &str = "{{ingress_hostname}}";
{% endif %}
{% if use_volume %}
const PVC_NAME: &str = "example";
const STORAGE_REQUEST: &str = "10Gi";
{% endif %}
fn labels() -> BTreeMap<String, String> {
    let mut labels: BTreeMap<String, String> = BTreeMap::new();
    labels.insert(String::from("app"), String::from(NAME));
    labels
}

fn create_deploy() -> anyhow::Result<Deployment> {
    let cpu_request = Quantity(String::from("25m"));
    let cpu_limit = Quantity(String::from("500m"));
    let memory = Quantity(String::from("64Mi"));

    let probe = http_probe("/example/health", IntOrString::String(String::from("app")), None, None, None, None);

    let container = ContainerBuilder::new()
        .name(NAME)
        .image(format!("{}:{}", IMAGE, VERSION))
        .container_port(PORT, "app", PortProtocol::TCP)
        .env("EXAMPLE_ENV_VAR", "ABC")
        .readiness_probe(probe.clone())
        .liveness_probe(probe)
        .cpu_request(cpu_request)
        .cpu_limit(cpu_limit)
        .memory_request(memory.clone())
        .memory_limit(memory)
        {% if use_volume %}
        .volume_mount(
            VolumeMount {
                name: String::from("example"),
                mount_path: String::from("/data/example"),
                read_only: Some(false),
                ..Default::default()
            }
        )
        {% endif %}
        .build()?;

    DeploymentBuilder::new()
        .name(NAME)
        .replicas(1)
        .selector_match_labels(labels())
        .pod_labels(labels())
        .container(container)
        {% if use_volume %}.volume_from_pvc("example", PVC_NAME){% endif %}
        .build()
}
{%if use_postgres %}
fn create_secret() -> anyhow::Result<Secret> {
    SecretBuilder::new()
        .name(DATABASE_SECRET)
        .value("database_host", PG_HOST.to_string().as_str())
        .value("database_port", PG_PORT.to_string().as_str())
        .value("database_db", DATABASE_NAME)
        .value("database_username", env!("POSTGRES_USERNAME"))
        .value("database_password", env!("POSTGRES_PASSWORD"))
        .build()
}
{% endif %}
{% if use_volume %}
fn create_pvc() -> anyhow::Result<PersistentVolumeClaim> {
    PersistentVolumeClaimBuilder::new()
        .name(PVC_NAME)
        .storage_class_name("linode-block-storage-retain-encrypted")
        .storage_requests(Quantity(String::from(STORAGE_REQUEST)))
        .build()
}
{% endif %}

fn create_service() -> anyhow::Result<Service> {
    ServiceBuilder::new()
        .selector(labels())
        .name(NAME)
        .port("app", PortProtocol::TCP, 80, PORT)
        .build()
}

{% if ingress_hostname != "" %}
fn create_ingress() -> anyhow::Result<Ingress> {
    IngressBuilder::new()
        .name(NAME)
        .annotation("cert-manager.io/cluster-issuer", "letsencrypt-prod")
        .ingress_class_name("nginx")
        .tls_host(HOSTNAME, NAME)
        .rule(HOSTNAME, "/", "Prefix", NAME, PORT)
        .build()
}
{% endif %}

fn main() -> anyhow::Result<()> {
    let deploy = create_deploy()?;
    let service = create_service()?;
    {% if use_postgres %}let secret = create_secret()?;{% endif %}
    {% if ingress_hostname != "" %}let ingress = create_ingress()?;{% endif %}
    {% if use_volume %}let pvc = create_pvc()?;{% endif %}

    let resources = vec![
        serde_json::value::to_value(deploy)?,
        serde_json::value::to_value(service)?,
        {% if use_postgres %}serde_json::value::to_value(secret)?,{% endif %}
        {% if ingress_hostname != "" %}serde_json::value::to_value(ingress)?,{% endif %}
        {% if use_volume %}serde_json::value::to_value(pvc)?,{% endif %}
    ];
    println!("{}", serde_json::to_string(&resources).unwrap());
    Ok(())
}

