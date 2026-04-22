use kube_builder::prelude::*;
use std::collections::BTreeMap;

use k8s_openapi::{
    api::{apps::v1::Deployment, core::v1::ServiceAccount, rbac::v1::{ClusterRole, ClusterRoleBinding, PolicyRule, RoleRef, Subject}},
    apimachinery::pkg::{api::resource::Quantity, apis::meta::v1::ObjectMeta},
};

const NAME: &str = "external-dns";
const IMAGE: &str = "registry.k8s.io/external-dns/external-dns";
const VERSION: &str = "v0.20.0";
const LINODE_SECRET_NAME: &str = "linode";
const NAMESPACE: &str = "external-dns";

fn create_deploy() -> anyhow::Result<Deployment> {
    let mut labels: BTreeMap<String, String> = BTreeMap::new();
    labels.insert(String::from("app"), String::from(NAME));

    let memory = Quantity(String::from("64Mi"));
    let container = ContainerBuilder::new()
        .name(NAME)
        .image(format!("{}:{}", IMAGE, VERSION))
        .env_from_secret("LINODE_TOKEN", LINODE_SECRET_NAME, "token")
        .cpu_request(Quantity(String::from("5m")))
        .cpu_limit(Quantity(String::from("200m")))
        .memory_request(memory.clone())
        .memory_limit(memory.clone())
        .args(vec![
            String::from("--source=ingress"),
            String::from("--domain-filter=bgottlob.com"),
            String::from("--provider=linode"),
            String::from("--registry=txt"),
            String::from("--txt-owner-id=bgottlob-k8s"),
            String::from("--txt-prefix=external-dns-"),
            String::from("--source=gateway-httproute"),
            String::from("--source=gateway-grpcroute"),
            String::from("--source=gateway-tlsroute"),
            String::from("--source=gateway-tcproute"),
            String::from("--source=gateway-udproute"),
        ])
        .build()?;

    DeploymentBuilder::new()
        .name(NAME)
        .replicas(1)
        .selector_match_labels(labels.clone())
        .pod_labels(labels.clone())
        .container(container)
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
           PolicyRule {
               api_groups: Some(vec![
                   String::from("discovery.k8s.io"),
               ]),
               resources: Some(vec![
                   String::from("endpointslices"),
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
                   String::from("namespaces"),
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
                   String::from("gateway.networking.k8s.io"),
               ]),
               resources: Some(vec![
                   String::from("gateways"),
                   String::from("httproutes"),
                   String::from("grpcroutes"),
                   String::from("tlsroutes"),
                   String::from("tcproutes"),
                   String::from("udproutes"),
               ]),
               verbs: vec![
                   String::from("get"),
                   String::from("watch"),
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
            namespace: Some(NAMESPACE.to_string()),
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
    let sealed_secrets = read_sealed_secrets_from_stdin()?;
    let deploy = create_deploy()?;
    let (cr, crb, sa) = create_rbac();

    let mut resources: Vec<Vec<serde_json::Value>> = Vec::new();
    if !sealed_secrets.is_empty() {
        resources.push(sealed_secrets);
    }
    resources.push(vec![
        serde_json::value::to_value(deploy)?,
        serde_json::value::to_value(cr)?,
        serde_json::value::to_value(crb)?,
        serde_json::value::to_value(sa)?,
    ]);
    println!("{}", serde_json::to_string(&resources)?);
    Ok(())
}
