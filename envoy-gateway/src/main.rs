use std::{collections::BTreeMap, env};
use k8s_gateway_api::prelude::*;
use kube::core::ObjectMeta;
use serde::Deserialize;

const GATEWAY_CLASS: &str = "envoy-public";

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

    println!("{}", serde_json::to_string(&resources).unwrap());
    Ok(())
}
