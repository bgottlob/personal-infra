use std::collections::BTreeMap;
use anyhow::anyhow;

use k8s_openapi::{api::{apps::v1::{Deployment, DeploymentSpec}, core::v1::{ConfigMapVolumeSource, Container, EnvVar, EnvVarSource, KeyToPath, LocalObjectReference, ObjectFieldSelector, PersistentVolumeClaimVolumeSource, PodSecurityContext, PodSpec, PodTemplateSpec, SecretKeySelector, SecretVolumeSource, SecurityContext, Volume}}, apimachinery::pkg::apis::meta::v1::{LabelSelector, ObjectMeta}};

#[derive(Default)]
pub struct DeploymentBuilder {
    name: Option<String>,
    replicas: Option<i32>,
    selector_match_labels: BTreeMap<String, String>,
    pod_labels: BTreeMap<String, String>,
    containers: Vec<Container>,
    volumes: Vec<Volume>,
    service_account_name: Option<String>,
    private_registry_pull_secret: bool,
    security_context: Option<PodSecurityContext>,
    automount_service_account_token: Option<bool>,
}

impl DeploymentBuilder {
    pub fn new() -> Self {
        DeploymentBuilder {
            private_registry_pull_secret: false,
            ..Default::default()
        }
    }

    pub fn service_account_name<S: Into<String>>(&mut self, sa: S) -> &mut Self {
        self.service_account_name = Some(sa.into());
        self
    }

    pub fn name<S: Into<String>>(&mut self, name: S) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    pub fn selector_match_labels(&mut self, mut labels: BTreeMap<String, String>) -> &mut Self {
        self.selector_match_labels.append(&mut labels);
        self
    }

    pub fn replicas(&mut self, replicas: i32) -> &mut Self {
        self.replicas = Some(replicas);
        self
    }

    pub fn pod_labels(&mut self, mut labels: BTreeMap<String, String>) -> &mut Self {
        self.pod_labels.append(&mut labels);
        self
    }

    pub fn container(&mut self, container: Container) -> &mut Self {
        self.containers.push(container);
        self
    }

    pub fn use_private_registry(&mut self) -> &mut Self {
        self.private_registry_pull_secret = true;
        self
    }

    pub fn automount_service_account_token(&mut self, enabled: bool) -> &mut Self {
        self.automount_service_account_token = Some(enabled);
        self
    }

    pub fn tailscale_container(&mut self) -> &mut Self {
        let container = Container {
            name: "ts-sidecar".into(),
            image: Some("ghcr.io/tailscale/tailscale:latest".into()),
            security_context: Some(SecurityContext {
                privileged: Some(true),
                ..Default::default()
            }),
            env: Some(vec![
                EnvVar {
                    name: "TS_KUBE_SECRET".into(),
                    value: Some("tailscale-auth".into()),
                    ..Default::default()
                },
                EnvVar {
                    name: "TS_USERSPACE".into(),
                    value: Some("false".into()),
                    ..Default::default()
                },
                EnvVar {
                    name: "TS_AUTHKEY".into(),
                    value_from: Some(EnvVarSource {
                        secret_key_ref: Some(SecretKeySelector {
                            key: "TS_AUTHKEY".into(),
                            name: "tailscale-auth".into(),
                            optional: Some(true)
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                EnvVar {
                    name: "POD_NAME".into(),
                    value_from: Some(EnvVarSource {
                        field_ref: Some(ObjectFieldSelector {
                            api_version: Some("v1".into()),
                            field_path: "metadata.name".into(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                EnvVar {
                    name: "POD_UID".into(),
                    value_from: Some(EnvVarSource {
                        field_ref: Some(ObjectFieldSelector {
                            api_version: Some("v1".into()),
                            field_path: "metadata.uid".into(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }
            ]),
            ..Default::default()
        };
        self.containers.push(container);
        self
    }

    pub fn volume_from_pvc<N: Into<String>, P: Into<String>>(&mut self, name: N, pvc_name: P) -> &mut Self {
        let volume = Volume {
            name: name.into(),
            persistent_volume_claim: Some(PersistentVolumeClaimVolumeSource {
                claim_name: pvc_name.into(),
                ..Default::default()
            }),
            ..Default::default()
        };
        self.volumes.push(volume);
        self
    }

    pub fn volume_from_configmap<N: Into<String>, C: Into<String>, K: Into<String>, P: Into<String>>(&mut self, name: N, cm_name: C, key: K, path: P) -> &mut Self {
        let volume = Volume {
            name: name.into(),
            config_map: Some(ConfigMapVolumeSource {
                name: cm_name.into(),
                items: Some(vec![KeyToPath {
                    key: key.into(),
                    path: path.into(),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };
        self.volumes.push(volume);
        self
    }

    pub fn volume_from_secret<N: Into<String>, S: Into<String>>(&mut self, name: N, secret_name: S) -> &mut Self {
        let volume = Volume {
            name: name.into(),
            secret: Some(SecretVolumeSource {
                secret_name: Some(secret_name.into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        self.volumes.push(volume);
        self
    }

    pub fn security_context(&mut self, pod_security_context: PodSecurityContext) -> &mut Self {
        self.security_context = Some(pod_security_context);
        self
    }

    pub fn build(&self) -> anyhow::Result<Deployment> {
        let name = self.name.clone().ok_or(anyhow!("Deployment must have a name"))?;

        let containers = match self.containers.is_empty() {
            false => Ok(self.containers.clone()),
            true => Err(anyhow!("At least one container must be added")),
        }?;

        let volumes = match self.containers.is_empty() {
            true => None,
            false => Some(self.volumes.clone()),
        };

        let image_pull_secrets = if self.private_registry_pull_secret {
            Some(vec![LocalObjectReference {
                name: "registry-creds".into()
            }])
        } else {
            None
        };

        let deployment = Deployment {
            metadata: ObjectMeta {
                name: Some(name),
                ..Default::default()
            },
            spec: Some(DeploymentSpec {
                selector: LabelSelector {
                    match_labels: Some(self.selector_match_labels.clone()),
                    ..Default::default()
                },
                replicas: Some(self.replicas.unwrap_or(1)),
                template: PodTemplateSpec {
                    metadata: Some(ObjectMeta {
                        labels: Some(self.pod_labels.clone()),
                        ..Default::default()
                    }),
                    spec: Some(PodSpec {
                        automount_service_account_token: self.automount_service_account_token,
                        security_context: self.security_context.clone(),
                        containers: containers,
                        volumes,
                        service_account_name: self.service_account_name.clone(),
                        image_pull_secrets,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }),
            ..Default::default()
        };

        Ok(deployment)
    }
}
