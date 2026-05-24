use std::collections::BTreeMap;
use std::env;

use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use serde::Deserialize;

mod tailscale;
use tailscale::proxy_class::*;

fn create_proxy_class() -> ProxyClass {
    ProxyClass {
        metadata: ObjectMeta {
            name: Some(String::from("default")),
            ..Default::default()
        },
        spec: ProxyClassSpec {
            stateful_set: Some(ProxyClassStatefulSet {
                pod: Some(ProxyClassStatefulSetPod {
                    tailscale_container: Some(ProxyClassStatefulSetPodTailscaleContainer {
                        resources: Some(ProxyClassStatefulSetPodTailscaleContainerResources {
                            requests: Some(BTreeMap::from([
                                (String::from("cpu"), IntOrString::String(String::from("5m"))),
                                (String::from("memory"), IntOrString::String(String::from("32Mi"))),
                            ])),
                            limits: Some(BTreeMap::from([
                                (String::from("cpu"), IntOrString::String(String::from("50m"))),
                                (String::from("memory"), IntOrString::String(String::from("64Mi"))),
                            ])),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn main() -> anyhow::Result<()> {
    let helm_out_str = include_str!(concat!(env!("OUT_DIR"), "/helm-output.yaml"));
    let mut resources: Vec<serde_json::Value> = Vec::new();
    for document in serde_norway::Deserializer::from_str(helm_out_str) {
        let value = serde_norway::Value::deserialize(document)?;
        if value != serde_norway::Value::Null {
            resources.push(serde_json::to_value(value)?);
        }
    }
    resources.push(serde_json::to_value(create_proxy_class())?);
    println!("{}", serde_json::to_string(&resources).unwrap());
    Ok(())
}
