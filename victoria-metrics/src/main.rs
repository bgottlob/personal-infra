mod vm;

use std::collections::BTreeMap;

use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::core::ObjectMeta;
use vm::vm_single::*;
use vm::vm_pod_scrape::*;
use vm::vm_node_scrape::*;
use vm::vm_agent::*;

use crate::vm::vm_pod_scrape::VMPodScrape;

fn create_vm() -> VMSingle {
    VMSingle {
        metadata: ObjectMeta {
            name: Some(String::from("main")),
            ..Default::default()
        },
        spec: VmSingleSpec {
            resources: Some(VmSingleResources {
                requests: Some(BTreeMap::from([
                                  (String::from("cpu"), IntOrString::String(String::from("100m"))),
                                  (String::from("memory"), IntOrString::String(String::from("128Mi"))),
                ])),
                limits: Some(BTreeMap::from([
                        (String::from("cpu"), IntOrString::String(String::from("1000m"))),
                        (String::from("memory"), IntOrString::String(String::from("500Mi"))),
                ])),
                ..Default::default()
            }),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn create_agent() -> VMAgent {
    VMAgent {
        metadata: ObjectMeta {
            name: Some(String::from("vmagent")),
            ..Default::default()
        },
        spec: VmAgentSpec {
            resources: Some(VmAgentResources {
                requests: Some(BTreeMap::from([
                  (String::from("cpu"), IntOrString::String(String::from("100m"))),
                  (String::from("memory"), IntOrString::String(String::from("128Mi"))),
                ])),
                limits: Some(BTreeMap::from([
                    (String::from("cpu"), IntOrString::String(String::from("1000m"))),
                    (String::from("memory"), IntOrString::String(String::from("500Mi"))),
                ])),
                ..Default::default()
            }),
            select_all_by_default: Some(true),
            replica_count: Some(1),
            scrape_interval: Some(String::from("30s")),
            scrape_timeout: Some(String::from("10s")),
            remote_write: vec![VmAgentRemoteWrite {
                url: String::from("http://vmsingle-main.victoria-metrics.svc.cluster.local:8428/api/v1/write"),
                ..Default::default()
            }],
            ..Default::default()
        },
        ..Default::default()
    }
}

fn create_kubelet_scrape() -> VMNodeScrape {
    VMNodeScrape {
        metadata: ObjectMeta {
            name: Some(String::from("kubelet")),
            ..Default::default()
        },
        spec: VmNodeScrapeSpec {
            scheme: Some(VmNodeScrapeScheme::Https),
            interval: Some(String::from("30s")),
            scrape_timeout: Some(String::from("5s")),
            tls_config: Some(VmNodeScrapeTlsConfig {
                insecure_skip_verify: Some(true),
                ca_file: Some(String::from("/var/run/secrets/kubernetes.io/serviceaccount/ca.crt")),
                ..Default::default()
            }),
            bearer_token_file: Some(String::from("/var/run/secrets/kubernetes.io/serviceaccount/token")),
            relabel_configs: Some(vec![
                VmNodeScrapeRelabelConfigs {
                    action: Some(String::from("labelmap")),
                    regex: Some(serde_json::Value::String(String::from("__meta_kubernetes_node_label_(.+)"))),
                    ..Default::default()
                },
                VmNodeScrapeRelabelConfigs {
                    target_label: Some(String::from("__address__")),
                    source_labels: Some(vec![
                        String::from("__address__")
                    ]),
                    regex: Some(serde_json::Value::String(String::from("([^:]+)(:[0-9]+)?"))),
                    replacement: Some(String::from("$1:10250")),
                    ..Default::default()
                },
                VmNodeScrapeRelabelConfigs {
                    target_label: Some(String::from("job")),
                    replacement: Some(String::from("kubelet")),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn create_cadvisor_scrape() -> VMNodeScrape {
    VMNodeScrape {
        metadata: ObjectMeta {
            name: Some(String::from("cadvisor")),
            ..Default::default()
        },
        spec: VmNodeScrapeSpec {
            scheme: Some(VmNodeScrapeScheme::Https),
            interval: Some(String::from("30s")),
            scrape_timeout: Some(String::from("5s")),
            tls_config: Some(VmNodeScrapeTlsConfig {
                insecure_skip_verify: Some(true),
                ca_file: Some(String::from("/var/run/secrets/kubernetes.io/serviceaccount/ca.crt")),
                ..Default::default()
            }),
            bearer_token_file: Some(String::from("/var/run/secrets/kubernetes.io/serviceaccount/token")),
            path: Some(String::from("/metrics/cadvisor")),
            relabel_configs: Some(vec![
                VmNodeScrapeRelabelConfigs {
                    action: Some(String::from("labelmap")),
                    regex: Some(serde_json::Value::String(String::from("__meta_kubernetes_node_label_(.+)"))),
                    ..Default::default()
                },
                VmNodeScrapeRelabelConfigs {
                    target_label: Some(String::from("__address__")),
                    source_labels: Some(vec![
                        String::from("__address__")
                    ]),
                    regex: Some(serde_json::Value::String(String::from("([^:]+)(:[0-9]+)?"))),
                    replacement: Some(String::from("$1:10250")),
                    ..Default::default()
                },
                VmNodeScrapeRelabelConfigs {
                    target_label: Some(String::from("job")),
                    replacement: Some(String::from("cadvisor")),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn create_all_pods_scrape() -> VMPodScrape {
    VMPodScrape {
        metadata: ObjectMeta {
            name: Some(String::from("all-scrape")),
            ..Default::default()
        },
        spec: VmPodScrapeSpec {
            pod_metrics_endpoints: vec![
               VmPodScrapePodMetricsEndpoints {
                   scheme: Some(VmPodScrapePodMetricsEndpointsScheme::Http),
                   relabel_configs: Some(vec![
                       VmPodScrapePodMetricsEndpointsRelabelConfigs {
                           source_labels: Some(vec![
                               String::from("__meta_kubernetes_pod_annotation_prometheus_io_scrape"),
                           ]),
                           action: Some(String::from("keep")),
                           regex: Some(serde_json::Value::String(String::from("true"))),
                           ..Default::default()
                       },
                       VmPodScrapePodMetricsEndpointsRelabelConfigs {
                           source_labels: Some(vec![
                               String::from("__meta_kubernetes_pod_annotation_prometheus_io_scheme"),
                           ]),
                           action: Some(String::from("replace")),
                           target_label: Some(String::from("__scheme__")),
                           regex: Some(serde_json::Value::String(String::from("(https?)"))),
                           ..Default::default()
                       },
                       VmPodScrapePodMetricsEndpointsRelabelConfigs {
                           source_labels: Some(vec![
                               String::from("__meta_kubernetes_pod_annotation_prometheus_io_path"),
                           ]),
                           action: Some(String::from("replace")),
                           target_label: Some(String::from("__metrics_path__")),
                           regex: Some(serde_json::Value::String(String::from("(.+)"))),
                           ..Default::default()
                       },
                       VmPodScrapePodMetricsEndpointsRelabelConfigs {
                           source_labels: Some(vec![
                               String::from("__address__"),
                               String::from("__meta_kubernetes_pod_annotation_prometheus_io_port"),
                           ]),
                           action: Some(String::from("replace")),
                           target_label: Some(String::from("__address__")),
                           regex: Some(serde_json::Value::String(String::from("([^:]+)(?::\\d+)?;(\\d+)"))),
                           replacement: Some(String::from("$1:$2")),
                           ..Default::default()
                       },
                   ]),
                   ..Default::default()
               },
            ],
            selector: Some(VmPodScrapeSelector::default()),
            namespace_selector: Some(VmPodScrapeNamespaceSelector {
                any: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn main() -> anyhow::Result<()> {
    let vm = create_vm();
    let agent = create_agent();
    let all_pods_scrape = create_all_pods_scrape();
    let cadvisor_scrape = create_cadvisor_scrape();
    let kubelet_scrape = create_kubelet_scrape();

    let mut resources: Vec<serde_json::Value> = Vec::new();
    let vm_value = serde_json::to_value(vm)?;
    let agent_value = serde_json::to_value(agent)?;
    let all_pods_scrape_value = serde_json::to_value(all_pods_scrape)?;
    let cadvisor_scrape_value = serde_json::to_value(cadvisor_scrape)?;
    let kubelet_scrape_value = serde_json::to_value(kubelet_scrape)?;

    resources.push(vm_value);
    resources.push(agent_value);
    resources.push(all_pods_scrape_value);
    resources.push(cadvisor_scrape_value);
    resources.push(kubelet_scrape_value);
    println!("{}", serde_json::to_string(&resources)?);

    Ok(())
}
