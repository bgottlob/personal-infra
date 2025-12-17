use kube_builder::prelude::*;
use std::collections::BTreeMap;

use k8s_openapi::{
    api::{apps::v1::Deployment, core::v1::{ResourceRequirements, Secret, ServiceAccount}, rbac::v1::{ClusterRole, ClusterRoleBinding, PolicyRule, RoleRef, Subject}},
    apimachinery::pkg::{api::resource::Quantity, apis::meta::v1::ObjectMeta},
};

const NAME: &str = "external-dns";
const IMAGE: &str = "registry.k8s.io/external-dns/external-dns";
const VERSION: &str = "v0.15.1";
const LINODE_SECRET_NAME: &str = "linode";
const NAMESPACE: &str = "external-dns";

fn create_secret() -> anyhow::Result<Secret> {
    SecretBuilder::new()
        .name(LINODE_SECRET_NAME)
        .value("token", env!("LINODE_TOKEN"))
        .build()
}

fn create_deploy() -> anyhow::Result<Deployment> {
    let mut labels: BTreeMap<String, String> = BTreeMap::new();
    labels.insert(String::from("app"), String::from(NAME));

    let env = EnvBuilder::new()
        .env_from_secret("LINODE_TOKEN", LINODE_SECRET_NAME, "token")
        .build();

    DeploymentBuilder::new()
        .name(NAME)
        .replicas(1)
        .selector_match_labels(labels.clone())
        .pod_labels(labels.clone())
        .container(
            NAME,
            format!("{}:{}", IMAGE, VERSION),
            "http",
            80, // TODO this doesn't actually need a port
            PortProtocol::TCP,
            env,
            None,
            Some(
                ResourceRequirements {
                    requests: Some(BTreeMap::from([
                        (String::from("cpu"), Quantity(String::from("25m"))),
                        (String::from("memory"), Quantity(String::from("128Mi"))),
                    ])),
                    ..Default::default()
                }
            ),
            None,
            Some(vec![
                String::from("--source=ingress"),
                String::from("--domain-filter=bgottlob.com"),
                String::from("--provider=linode"),
                String::from("--registry=txt"),
                String::from("--txt-owner-id=bgottlob-k8s"),
                String::from("--txt-prefix=external-dns-"),
            ])
        )
        .build()
}

fn create_rbac() -> (ClusterRole, ClusterRoleBinding, ServiceAccount) {
    let role_name = NAME;
    let sa_name = NAME;

    let cr = ClusterRole {
        metadata: ObjectMeta {
            name: Some(role_name.to_string()),
            ..Default::default()
        },
        rules: Some(vec![
           PolicyRule {
               api_groups: Some(vec![String::from("")]),
               resources: Some(vec![
                   String::from("services"),
                   String::from("endpoints"),
                   String::from("pods"),
               ]),
               verbs: vec![
                   String::from("get"),
                   String::from("watch"),
                   String::from("list"),
               ],
               ..Default::default()
           },
           PolicyRule {
               api_groups: Some(vec![
                   String::from("extensions"),
                   String::from("networking.k8s.io"),
               ]),
               resources: Some(vec![
                   String::from("ingresses"),
               ]),
               verbs: vec![
                   String::from("get"),
                   String::from("watch"),
                   String::from("list"),
               ],
               ..Default::default()
           },
           PolicyRule {
               api_groups: Some(vec![
                   String::from(""),
               ]),
               resources: Some(vec![
                   String::from("nodes"),
               ]),
               verbs: vec![
                   String::from("list"),
               ],
               ..Default::default()
           },
        ]),
        ..Default::default()
    };

    let crb = ClusterRoleBinding {
        metadata: ObjectMeta {
            name: Some(format!("{}-viewer", role_name)),
            ..Default::default()
        },
        role_ref: RoleRef {
            kind: String::from("ClusterRole"),
            name: role_name.to_string(),
            ..Default::default()
        },
        subjects: Some(vec![Subject {
            name: sa_name.to_string(),
            kind: String::from("ServiceAccount"),
            namespace: Some(String::from("ServiceAccount")),
            ..Default::default()
        }]),
    };

    let sa = ServiceAccount {
        metadata: ObjectMeta {
            name: Some(sa_name.to_string()),
            namespace: Some(NAMESPACE.to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    (cr, crb, sa)
}

fn main() -> anyhow::Result<()> {
    let deploy = create_deploy()?;
    let secret = create_secret()?;
    let (cr, crb, sa) = create_rbac();

    let resources = vec![
        serde_json::value::to_value(deploy)?,
        serde_json::value::to_value(secret)?,
        serde_json::value::to_value(cr)?,
        serde_json::value::to_value(crb)?,
        serde_json::value::to_value(sa)?,
    ];
    println!("{}", serde_json::to_string(&resources).unwrap());
    Ok(())
}
