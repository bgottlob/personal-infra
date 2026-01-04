use k8s_openapi::api::core::v1::{Capabilities, HTTPGetAction, Probe, SeccompProfile, SecurityContext, ServiceAccount};
use k8s_openapi::api::rbac::v1::{ClusterRole, ClusterRoleBinding, PolicyRule, RoleRef, Subject};
use kube::core::ObjectMeta;
use kube_builder::prelude::*;

use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use k8s_openapi::{
    api::{
        apps::v1::Deployment,
        core::v1::Service,
    },
    apimachinery::pkg::api::resource::Quantity,
};
use std::collections::BTreeMap;

const NAME: &str = "kube-state-metrics";
const VERSION: &str = "2.17.0";
const IMAGE: &str = "registry.k8s.io/kube-state-metrics/kube-state-metrics";

const METRICS_PORT_NAME: &str = "http-metrics";
const METRICS_PORT: i32 = 8080;
const TELEMETRY_PORT_NAME: &str = "telemetry";
const TELEMETRY_PORT: i32 = 8081;


fn create_rbac() -> (ClusterRole, ClusterRoleBinding, ServiceAccount) {
    let crb = ClusterRoleBinding {
        metadata: ObjectMeta {
            name: Some(NAME.to_string()),
            ..Default::default()
        },
        role_ref: RoleRef {
            api_group: String::from("rbac.authorization.k8s.io"),
            kind: String::from("ClusterRole"),
            name: String::from(NAME.to_string()),
        },
        subjects: Some(vec![
              Subject {
                  kind: String::from("ServiceAccount"),
                  name: NAME.to_string(),
                  namespace: Some(String::from("kube-system")),
                  ..Default::default()
              }
        ])
    };

    let role = ClusterRole {
        metadata: ObjectMeta {
            name: Some(NAME.to_string()),
            ..Default::default()
        },
        rules: Some(vec![
           PolicyRule {
               api_groups: Some(vec![String::from("")]),
               resources: Some(vec![
                   String::from("configmaps"),
                   String::from("secrets"),
                   String::from("nodes"),
                   String::from("pods"),
                   String::from("services"),
                   String::from("serviceaccounts"),
                   String::from("resourcequotas"),
                   String::from("replicationcontrollers"),
                   String::from("limitranges"),
                   String::from("persistentvolumeclaims"),
                   String::from("persistentvolumes"),
                   String::from("namespaces"),
                   String::from("endpoints"),
               ]),
               verbs: vec![
                   String::from("list"),
                   String::from("watch"),
               ],
               ..Default::default()
           },
           PolicyRule {
               api_groups: Some(vec![String::from("apps")]),
               resources: Some(vec![
                   String::from("statefulsets"),
                   String::from("daemonsets"),
                   String::from("deployments"),
                   String::from("replicasets"),
               ]),
               verbs: vec![
                   String::from("list"),
                   String::from("watch"),
               ],
               ..Default::default()
           },
           PolicyRule {
               api_groups: Some(vec![String::from("batch")]),
               resources: Some(vec![
                   String::from("cronjobs"),
                   String::from("jobs"),
               ]),
               verbs: vec![
                   String::from("list"),
                   String::from("watch"),
               ],
               ..Default::default()
           },
           PolicyRule {
               api_groups: Some(vec![String::from("autoscaling")]),
               resources: Some(vec![
                   String::from("horizontalpodautoscalers"),
               ]),
               verbs: vec![
                   String::from("list"),
                   String::from("watch"),
               ],
               ..Default::default()
           },
           PolicyRule {
               api_groups: Some(vec![String::from("authetication.k8s.io")]),
               resources: Some(vec![
                   String::from("tokenreviews"),
               ]),
               verbs: vec![
                   String::from("create"),
               ],
               ..Default::default()
           },
           PolicyRule {
               api_groups: Some(vec![String::from("authorization.k8s.io")]),
               resources: Some(vec![
                   String::from("subjectaccessreviews"),
               ]),
               verbs: vec![
                   String::from("create"),
               ],
               ..Default::default()
           },
           PolicyRule {
               api_groups: Some(vec![String::from("policy")]),
               resources: Some(vec![
                   String::from("poddisruptionbudgets"),
               ]),
               verbs: vec![
                   String::from("list"),
                   String::from("watch"),
               ],
               ..Default::default()
           },
           PolicyRule {
               api_groups: Some(vec![String::from("certificates.k8s.io")]),
               resources: Some(vec![
                   String::from("certificatesigningrequests"),
               ]),
               verbs: vec![
                   String::from("list"),
                   String::from("watch"),
               ],
               ..Default::default()
           },
           PolicyRule {
               api_groups: Some(vec![String::from("discovery.k8s.io")]),
               resources: Some(vec![
                   String::from("endpointslices"),
               ]),
               verbs: vec![
                   String::from("list"),
                   String::from("watch"),
               ],
               ..Default::default()
           },
           PolicyRule {
               api_groups: Some(vec![String::from("storage.k8s.io")]),
               resources: Some(vec![
                   String::from("storageclasses"),
                   String::from("volumeattachments"),
               ]),
               verbs: vec![
                   String::from("list"),
                   String::from("watch"),
               ],
               ..Default::default()
           },
           PolicyRule {
               api_groups: Some(vec![String::from("admissionregistration.k8s.io")]),
               resources: Some(vec![
                   String::from("mutatingwebhookconfigurations"),
                   String::from("validatingwebhookconfigurations"),
               ]),
               verbs: vec![
                   String::from("list"),
                   String::from("watch"),
               ],
               ..Default::default()
           },
           PolicyRule {
               api_groups: Some(vec![String::from("networking.k8s.io")]),
               resources: Some(vec![
                   String::from("networkpolicies"),
                   String::from("ingressclasses"),
                   String::from("ingresses"),
               ]),
               verbs: vec![
                   String::from("list"),
                   String::from("watch"),
               ],
               ..Default::default()
           },
           PolicyRule {
               api_groups: Some(vec![String::from("coordination.k8s.io")]),
               resources: Some(vec![
                   String::from("leases"),
               ]),
               verbs: vec![
                   String::from("list"),
                   String::from("watch"),
               ],
               ..Default::default()
           },
           PolicyRule {
               api_groups: Some(vec![String::from("rbac.authorization.k8s.io")]),
               resources: Some(vec![
                   String::from("clusterrolebindings"),
                   String::from("clusterroles"),
                   String::from("rolebindings"),
                   String::from("roles"),
               ]),
               verbs: vec![
                   String::from("list"),
                   String::from("watch"),
               ],
               ..Default::default()
           },
        ]),
        ..Default::default()
    };

    let sa = ServiceAccount {
        automount_service_account_token: Some(false),
        metadata: ObjectMeta {
            name: Some(NAME.to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    (role, crb, sa)
}

fn labels() -> BTreeMap<String, String> {
    let mut labels: BTreeMap<String, String> = BTreeMap::new();
    labels.insert(String::from("app"), String::from(NAME));
    labels
}

fn create_deploy() -> anyhow::Result<Deployment> {
    let cpu_request = Quantity(String::from("35m"));
    let cpu_limit = Quantity(String::from("200m"));
    let memory = Quantity(String::from("64Mi"));

    let readiness_probe = Probe {
        http_get: Some(HTTPGetAction {
            path: Some(String::from("/readyz")),
            port: IntOrString::String(TELEMETRY_PORT_NAME.to_string()),
            ..Default::default()
        }),
        initial_delay_seconds: Some(5),
        timeout_seconds: Some(5),
        ..Default::default()
    };

    let container_security_context = SecurityContext {
        allow_privilege_escalation: Some(true),
        capabilities: Some(Capabilities {
            drop: Some(vec![String::from("ALL")]),
            ..Default::default()
        }),
        read_only_root_filesystem: Some(true),
        run_as_non_root: Some(true),
        run_as_user: Some(65534),
        seccomp_profile: Some(SeccompProfile {
            type_: String::from("RuntimeDefault"),
            ..Default::default()
        }),
        ..Default::default()
    };

    let container = ContainerBuilder::new()
        .name(NAME)
        .image(format!("{}:v{}", IMAGE, VERSION))
        .container_port(METRICS_PORT, METRICS_PORT_NAME, PortProtocol::TCP)
        .container_port(TELEMETRY_PORT, TELEMETRY_PORT_NAME, PortProtocol::TCP)
        .readiness_probe(readiness_probe)
        .cpu_request(cpu_request)
        .cpu_limit(cpu_limit)
        .memory_request(memory.clone())
        .memory_limit(memory)
        .security_context(container_security_context)
        .build()?;

    DeploymentBuilder::new()
        .name(NAME)
        .replicas(1)
        .selector_match_labels(labels())
        .pod_labels(labels())
        .container(container)
        .service_account_name(NAME)
        .automount_service_account_token(true)
        .build()
}

fn create_service() -> anyhow::Result<Service> {
    ServiceBuilder::new()
        .label("vm_target", NAME)
        .selector(labels())
        .name(NAME)
        .headless()
        .port(METRICS_PORT_NAME, PortProtocol::TCP, METRICS_PORT, METRICS_PORT)
        .port(TELEMETRY_PORT_NAME, PortProtocol::TCP, TELEMETRY_PORT, TELEMETRY_PORT)
        .build()
}

fn main() -> anyhow::Result<()> {
    let deploy = create_deploy()?;
    let service = create_service()?;
    let (cr, crb, sa) = create_rbac();

    let resources = vec![
        serde_json::value::to_value(deploy)?,
        serde_json::value::to_value(service)?,
        serde_json::value::to_value(cr)?,
        serde_json::value::to_value(crb)?,
        serde_json::value::to_value(sa)?,
    ];
    println!("{}", serde_json::to_string(&resources).unwrap());
    Ok(())
}
