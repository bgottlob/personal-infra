use std::env;
use kube::core::ObjectMeta;
use serde::Deserialize;

mod cluster_issuers;
use cluster_issuers::*;

fn main() -> anyhow::Result<()> {
    let helm_out_str = include_str!(concat!(env!("OUT_DIR"), "/helm-output.yaml"));
    let mut resources: Vec<serde_json::Value> = Vec::new();
    for document in serde_norway::Deserializer::from_str(helm_out_str) {
        let value = serde_norway::Value::deserialize(document)?;
        if value != serde_norway::Value::Null {
            resources.push(serde_json::to_value(value)?);
        }
    }
    // TODO The ClusterIssuer has to be built after the rest of the release,
    // since it relies on the cert-manager webhooks. Is it possible to apply
    // this in a second step?
    resources.push(serde_json::to_value(create_cluster_issuer())?);
    println!("{}", serde_json::to_string(&resources).unwrap());
    Ok(())
}

fn create_cluster_issuer() -> ClusterIssuer {
    let issuer_name = String::from("letsencrypt-prod");
    let ingress_class = String::from("nginx");

    ClusterIssuer {
        metadata: ObjectMeta {
            name: Some(issuer_name.clone()),
            ..Default::default()
        },
        spec: ClusterIssuerSpec {
            acme: Some(ClusterIssuerAcme {
                email: Some(String::from("info@bgottlob.com")),
                server: String::from("https://acme-v02.api.letsencrypt.org/directory"),
                private_key_secret_ref: ClusterIssuerAcmePrivateKeySecretRef {
                    name: format!("{}-secret", issuer_name),
                    ..Default::default()
                },
                solvers: Some(vec![ClusterIssuerAcmeSolvers {
                    http01: Some(ClusterIssuerAcmeSolversHttp01 {
                        ingress: Some(ClusterIssuerAcmeSolversHttp01Ingress {
                            class: Some(ingress_class),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        },
        ..Default::default()
    }
}
