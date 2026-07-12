use std::{collections::BTreeMap, env};
use k8s_gateway_api::prelude::*;
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::core::ObjectMeta;
use serde::Deserialize;


mod envoy;
use envoy::envoy_proxy::*;

const GATEWAY_CLASS: &str = "envoy-public";

fn create_envoy_proxy() -> EnvoyProxy {
    EnvoyProxy {
        metadata: ObjectMeta {
            name: Some(String::from("proxy-config")),
            ..Default::default()
        },
        spec: EnvoyProxySpec {
            provider: Some(EnvoyProxyProvider {
                r#type: EnvoyProxyProviderType::Kubernetes,
                kubernetes: Some(EnvoyProxyProviderKubernetes {
                    envoy_deployment: Some(EnvoyProxyProviderKubernetesEnvoyDeployment {
                        container: Some(EnvoyProxyProviderKubernetesEnvoyDeploymentContainer {
                            resources: Some(EnvoyProxyProviderKubernetesEnvoyDeploymentContainerResources {
                                requests: Some(BTreeMap::from([
                                  (String::from("cpu"), IntOrString::String(String::from("10m"))),
                                  (String::from("memory"), IntOrString::String(String::from("32Mi")))
                                ])),
                                limits: Some(BTreeMap::from([
                                        (String::from("cpu"), IntOrString::String(String::from("500m"))),
                                        (String::from("memory"), IntOrString::String(String::from("256Mi")))
                                ])),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    // Proxy Protocol v2 makes the Linode NodeBalancer forward the real client IP
                    // instead of its own internal address, which is required for geo lookups.
                    envoy_service: Some(EnvoyProxyProviderKubernetesEnvoyService {
                        annotations: Some(BTreeMap::from([
                            (
                                String::from("service.beta.kubernetes.io/linode-loadbalancer-proxy-protocol"),
                                String::from("v2"),
                            ),
                        ])),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                host: None,
            }),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn create_gateway() -> Gateway {
    Gateway {
        metadata: ObjectMeta {
            name: Some(GATEWAY_CLASS.to_string()),
            ..Default::default()
        },
        spec: GatewaySpec {
            gateway_class_name: GATEWAY_CLASS.to_string(),
            listeners: vec![
                GatewayListeners {
                    name: String::from("http"),
                    protocol: String::from("HTTP"),
                    port: 80,
                    ..Default::default()
                },
            ],
            allowed_listeners: Some(GatewayAllowedListeners {
                namespaces: Some(GatewayAllowedListenersNamespaces {
                    from: Some(GatewayAllowedListenersNamespacesFrom::All),
                    ..Default::default()
                }),
            }),
            infrastructure: Some(GatewayInfrastructure {
                parameters_ref: Some(GatewayInfrastructureParametersRef {
                    group: String::from("gateway.envoyproxy.io"),
                    kind: String::from("EnvoyProxy"),
                    name: String::from("proxy-config"),
                }),
                ..Default::default()
            }),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn create_gateway_class() -> GatewayClass {
    GatewayClass {
        metadata: ObjectMeta {
            name: Some(GATEWAY_CLASS.to_string()),
            ..Default::default()
        },
        spec: GatewayClassSpec {
            controller_name: env!("GATEWAY_CONTROLLER_NAME").to_string(),
            ..Default::default()
        },
        ..Default::default()
    }
}

// Proxy Protocol v2: NodeBalancer sends real client IP via proxy protocol headers;
// this policy tells Envoy to parse them.
fn create_client_traffic_policy() -> serde_json::Value {
    serde_json::json!({
        "apiVersion": "gateway.envoyproxy.io/v1alpha1",
        "kind": "ClientTrafficPolicy",
        "metadata": {
            "name": "proxy-protocol",
            "namespace": "envoy-gateway-system"
        },
        "spec": {
            "enableProxyProtocol": true,
            "targetRef": {
                "group": "gateway.networking.k8s.io",
                "kind": "Gateway",
                "name": GATEWAY_CLASS
            }
        }
    })
}

fn main() -> anyhow::Result<()> {
    let mut resources: Vec<serde_json::Value> = Vec::new();

    let main_helm_out_str = include_str!(concat!(env!("OUT_DIR"), "/main-helm-output.yaml"));
    for document in serde_norway::Deserializer::from_str(main_helm_out_str) {
        let value = serde_norway::Value::deserialize(document)?;
        if value != serde_norway::Value::Null {
            resources.push(serde_json::to_value(value)?);
        }
    }

    let crds_helm_out_str = include_str!(concat!(env!("OUT_DIR"), "/crds-helm-output.yaml"));
    for document in serde_norway::Deserializer::from_str(crds_helm_out_str) {
        let value = serde_norway::Value::deserialize(document)?;
        if value != serde_norway::Value::Null {
            resources.push(serde_json::to_value(value)?);
        }
    }

    let gc = create_gateway_class();
    resources.push(serde_json::to_value(&gc)?);
    let gateway = create_gateway();
    resources.push(serde_json::to_value(&gateway)?);
    let proxy = create_envoy_proxy();
    resources.push(serde_json::to_value(&proxy)?);
    resources.push(create_client_traffic_policy());

    println!("{}", serde_json::to_string(&resources).unwrap());
    Ok(())
}
