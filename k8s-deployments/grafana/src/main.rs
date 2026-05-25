use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::{ConfigMapVolumeSource, KeyToPath, PersistentVolumeClaimVolumeSource, PodSpec, PodTemplateSpec, Volume};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector;
use k8s_openapi::{api::{apps::v1::{StatefulSet, StatefulSetSpec}, core::v1::{ConfigMap, HTTPGetAction, PersistentVolumeClaim, PodSecurityContext, Probe, Service, TCPSocketAction, VolumeMount}}, apimachinery::pkg::{api::resource::Quantity, util::intstr::IntOrString}}; use kube::core::ObjectMeta;
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

fn create_statefulset() -> anyhow::Result<StatefulSet> {
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
        .memory_limit(Quantity(String::from("1Gi")))
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

    let sts = StatefulSet {
        metadata: ObjectMeta {
            name: Some(NAME.to_string()),
            ..Default::default()
        },
        spec: Some(StatefulSetSpec {
            selector: LabelSelector {
                match_labels: Some(labels()),
                ..Default::default()
            },
            replicas: Some(1),
            template: PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    labels: Some(labels()),
                    ..Default::default()
                }),
                spec: Some(PodSpec {
                    volumes: Some(vec![
                        Volume {
                            name: String::from("datasources"),
                            config_map: Some(ConfigMapVolumeSource {
                                name: String::from("datasources"),
                                items: Some(vec![KeyToPath {
                                    key: String::from("victoriametrics.yaml"),
                                    path: String::from("victoriametrics.yaml"),
                                    ..Default::default()
                                }]),
                                ..Default::default()
                            }),
                            ..Default::default()
                        },
                        Volume {
                            name: String::from("grafana"),
                            persistent_volume_claim: Some(PersistentVolumeClaimVolumeSource {

                                claim_name: NAME.to_string(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        },
                    ]),
                    containers: vec![container],
                    security_context: Some(PodSecurityContext {
                        fs_group: Some(472),
                        supplemental_groups: Some(vec![0]),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        }),
        ..Default::default()
    };

    Ok(sts)
}

fn create_service() -> anyhow::Result<Service> {
    ServiceBuilder::new()
        .name(NAME)
        .port(CONTAINER_PORT_NAME, PortProtocol::TCP, 80, PORT)
        .selector(labels())
        .expose_to_tailnet(Some(NAME))
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
    let sts = create_statefulset()?;
    let cm = create_configmap()?;

    resources.push(serde_json::to_value(&pvc)?);
    resources.push(serde_json::to_value(&svc)?);
    resources.push(serde_json::to_value(&sts)?);
    resources.push(serde_json::to_value(&cm)?);

    println!("{}", serde_json::to_string(&resources)?);

    Ok(())
}
