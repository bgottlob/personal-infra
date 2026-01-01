use std::collections::BTreeMap;

use k8s_openapi::{api::{apps::v1::Deployment, core::v1::{ConfigMap, HTTPGetAction, PersistentVolumeClaim, PodSecurityContext, Probe, Service, TCPSocketAction, VolumeMount}, networking::v1::Ingress}, apimachinery::pkg::{api::resource::Quantity, util::intstr::IntOrString}};
use kube::core::ObjectMeta;
use kube_builder::prelude::*;
use serde_json::json;

const NAME: &str = "grafana";
const IMAGE: &str = "grafana/grafana";
const VERSION: &str = "12.3.1";
const PORT: i32 = 3000;
const CONTAINER_PORT_NAME: &str = "http-grafana";

fn labels() -> BTreeMap<String, String> {
    BTreeMap::from([
        (String::from("app"), NAME.to_string()),
    ])
}

fn create_deployment() -> anyhow::Result<Deployment> {
    let container = ContainerBuilder::new()
        .name(NAME)
        .image(format!("{}:{}", IMAGE, VERSION))
        .container_port(PORT, CONTAINER_PORT_NAME, PortProtocol::TCP)
        .readiness_probe(Probe {
            failure_threshold: Some(3),
            http_get: Some(HTTPGetAction {
                path: Some(String::from("/robots.txt")),
                port: IntOrString::Int(PORT),
                scheme: Some(String::from("HTTP")),
                ..Default::default()
            }),
            ..Default::default()
        })
        .liveness_probe(Probe {
            failure_threshold: Some(3),
            initial_delay_seconds: Some(30),
            period_seconds: Some(10),
            success_threshold: Some(1),
            tcp_socket: Some(TCPSocketAction {
                port: IntOrString::Int(PORT),
                ..Default::default()
            }),
            ..Default::default()
        })
        .env("GF_INSTALL_PLUGINS", "victoriametrics-metrics-datasource")
        .cpu_request(Quantity(String::from("250m")))
        .cpu_limit(Quantity(String::from("500m")))
        .memory_request(Quantity(String::from("256Mi"))) // recommended is 750Mi
        .memory_limit(Quantity(String::from("256Mi")))
        .volume_mount(VolumeMount {
            mount_path: String::from("/var/lib/grafana"),
            name: String::from("grafana"),
            ..Default::default()
        })
        .volume_mount(VolumeMount {
            mount_path: String::from("/etc/grafana/provisioning/datasources"),
            name: String::from("datasources"),
            read_only: Some(true),
            ..Default::default()
        })
        .build()?;

    DeploymentBuilder::new()
        .name(NAME)
        .selector_match_labels(labels())
        .pod_labels(labels())
        .volume_from_pvc("grafana", NAME)
        .container(container)
        .security_context(PodSecurityContext {
            fs_group: Some(472),
            supplemental_groups: Some(vec![0]),
            ..Default::default()
        })
        .volume_from_configmap("datasources", "datasources", "victoriametrics.yaml", "victoriametrics.yaml")
        .build()
}

fn create_service() -> anyhow::Result<Service> {
    ServiceBuilder::new()
        .name(NAME)
        .port(CONTAINER_PORT_NAME, PortProtocol::TCP, 80, PORT)
        .selector(labels())
        .build()
}

fn create_ingress() -> anyhow::Result<Ingress> {
    IngressBuilder::new()
        .name(NAME)
        .ingress_class_name("nginx")
        .tls_host("grafana.bgottlob.com", NAME)
        .rule("grafana.bgottlob.com", "/", "Prefix", NAME, 3000)
        .annotation("cert-manager.io/cluster-issuer", "letsencrypt-prod")
        .build()
}

fn create_pvc() -> anyhow::Result<PersistentVolumeClaim> {
    PersistentVolumeClaimBuilder::new()
        .name(NAME)
        .storage_requests(Quantity(String::from("1Gi")))
        .storage_class_name("linode-block-storage-retain-encrypted")
        .build()
}

fn create_configmap() -> anyhow::Result<ConfigMap> {
    let data = json!({
        "apiVersion": 1,
        "datasources": [
            {
                "name": "VictoriaMetrics",
                "type": "victoriametrics-metrics-datasource",
                "access": "proxy",
                "url": "http://vmsingle-main.victoria-metrics:8428",
                "isDefault": true
            },
        ]
    });
    let data_str = serde_norway::to_string(&data)?;

    let data_map = BTreeMap::from([
        (String::from("victoriametrics.yaml"), data_str)
    ]);

    let cm = ConfigMap {
        metadata: ObjectMeta {
            name: Some(String::from("datasources")),
            ..Default::default()
        },
        data: Some(data_map),
        ..Default::default()
    };
    Ok(cm)
}

fn main() -> anyhow::Result<()> {
    let mut resources = Vec::new();

    let pvc = create_pvc()?;
    let svc = create_service()?;
    let ing = create_ingress()?;
    let deployment = create_deployment()?;
    let cm = create_configmap()?;

    let pvc_value = serde_json::to_value(&pvc)?;
    let svc_value = serde_json::to_value(&svc)?;
    let ing_value = serde_json::to_value(&ing)?;
    let deployment_value = serde_json::to_value(&deployment)?;
    let cm_value = serde_json::to_value(&cm)?;

    resources.push(pvc_value);
    resources.push(svc_value);
    resources.push(ing_value);
    resources.push(deployment_value);
    resources.push(cm_value);

    println!("{}", serde_json::to_string(&resources)?);

    Ok(())
}
