/*
 * This module is equivalent to the steps documented here:
 * https://tailscale.com/kb/1185/kubernetes#prerequisites
 */

use k8s_openapi::{api::{core::v1::ServiceAccount, rbac::v1::{PolicyRule, Role, RoleBinding, RoleRef, Subject}}, apimachinery::pkg::apis::meta::v1::ObjectMeta};

const SA_NAME: &str = "tailscale";
const TS_KUBE_SECRET: &str = "tailscale-auth";

fn service_account() -> ServiceAccount {
    ServiceAccount {
        metadata: ObjectMeta {
            name: Some(SA_NAME.into()),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn role() -> Role {
    Role {
        metadata: ObjectMeta {
            name: Some(SA_NAME.into()),
            ..Default::default()
        },
        rules: Some(vec![
           PolicyRule {
               api_groups: Some(vec!["".into()]),
               resources: Some(vec!["secrets".into()]),
               verbs: vec!["create".into()],
               ..Default::default()
           },
           PolicyRule {
               api_groups: Some(vec!["".into()]),
               resource_names: Some(vec![TS_KUBE_SECRET.into()]),
               resources: Some(vec!["secrets".into()]),
               verbs: vec!["get".into(), "update".into(), "patch".into()],
               ..Default::default()
           },
           PolicyRule {
               api_groups: Some(vec!["".into()]),
               resources: Some(vec!["events".into()]),
               verbs: vec!["get".into(), "create".into(), "patch".into()],
               ..Default::default()
           },
        ])
    }
}

fn role_binding() -> RoleBinding {
    RoleBinding {
        metadata: ObjectMeta {
            name: Some(SA_NAME.into()),
            ..Default::default()
        },
        subjects: Some(vec![Subject {
            kind: "ServiceAccount".into(),
            name: SA_NAME.into(),
            ..Default::default()
        }]),
        role_ref: RoleRef {
            kind: "Role".into(),
            name: SA_NAME.into(),
            api_group: "rbac.authorization.k8s.io".into(),
            ..Default::default()
        }
    }
}

pub fn rbac() -> (Role, RoleBinding, ServiceAccount) {
    (role(), role_binding(), service_account())
}
