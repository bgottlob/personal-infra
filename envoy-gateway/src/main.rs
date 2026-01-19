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
    let mut listeners: Vec<GatewayListeners> = [
        ("blog", "bgottlob.com"),
        ("grafana", "grafana.bgottlob.com"),
        ("kavita", "library.bgottlob.com"),
        ("miniflux", "miniflux.bgottlob.com"),
        ("registry", "registry.bgottlob.com"),
        ("rmfakecloud", "remarkable.bgottlob.com"),
        ("wallabag", "wallabag.bgottlob.com"),
    ]
        .iter()
        .map(|(name, hostname)| {
            GatewayListeners {
                allowed_routes: Some(GatewayListenersAllowedRoutes {
                    namespaces: Some(GatewayListenersAllowedRoutesNamespaces {
                        from: Some(GatewayListenersAllowedRoutesNamespacesFrom::All),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                name: format!("{}-https", name),
                protocol: String::from("HTTPS"),
                port: 443,
                hostname: Some(hostname.to_string()),
                tls: Some(GatewayListenersTls {
                    mode: Some(GatewayListenersTlsMode::Terminate),
                    certificate_refs: Some(
                        vec![GatewayListenersTlsCertificateRefs {
                            kind: Some(String::from("Secret")),
                            name: format!("{}-https", name),
                            ..Default::default()
                        }]
                    ),
                    ..Default::default()
                }),
                ..Default::default()
            }
        })
    .collect();

    listeners.push(
        GatewayListeners {
            name: String::from("http"),
            protocol: String::from("HTTP"),
            port: 80,
            ..Default::default()
        },
    );

    Gateway {
        metadata: ObjectMeta {
            annotations: Some(BTreeMap::from([
                (String::from("cert-manager.io/cluster-issuer"), String::from("letsencrypt-prod")),
            ])),
            name: Some(GATEWAY_CLASS.to_string()),
            ..Default::default()
        },
        spec: GatewaySpec {
            gateway_class_name: GATEWAY_CLASS.to_string(),
            listeners,
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

    println!("{}", serde_json::to_string(&resources).unwrap());
    Ok(())
}
