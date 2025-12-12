use std::collections::BTreeMap;
use anyhow::anyhow;

use k8s_openapi::{api::{apps::v1::{Deployment, DeploymentSpec}, core::v1::{Container, ContainerPort, EnvVar, EnvVarSource, HTTPGetAction, LocalObjectReference, ObjectFieldSelector, PersistentVolumeClaimVolumeSource, PodSpec, PodTemplateSpec, Probe, ResourceRequirements, SecretKeySelector, SecurityContext, Volume}}, apimachinery::pkg::{apis::meta::v1::{LabelSelector, ObjectMeta}, util::intstr::IntOrString}};

use crate::PortProtocol;

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

    pub fn container<
        N: Into<String>,
        I: Into<String>,
        P: Into<String>,
    >(&mut self, name: N, image: I, port_name: P, port: i32, port_protocol: PortProtocol, env: Vec<EnvVar>, liveness_probe: Option<Probe>, resources: Option<ResourceRequirements>) -> &mut Self {
        let container = Container {
            name: name.into(),
            image: Some(image.into()),
            ports: Some(vec![ContainerPort {
                name: Some(port_name.into()),
                container_port: port,
                protocol: Some(port_protocol.to_string()),
                ..Default::default()
            }]),
            env: Some(env),
            liveness_probe,
            resources,
            ..Default::default()
        };
        self.containers.push(container);
        self
    }

    pub fn use_private_registry(&mut self) -> &mut Self {
        self.private_registry_pull_secret = true;
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

#[derive(Default)]
pub struct EnvBuilder {
    vars: Vec<EnvVar>,
}

impl EnvBuilder {
    pub fn new() -> EnvBuilder {
        EnvBuilder::default()
    }

    pub fn env<N: Into<String>, V: Into<String>>(&mut self, name: N, value: V) -> &mut Self {
        let var = EnvVar {
            name: name.into(),
            value: Some(value.into()),
            ..Default::default()
        };
        self.vars.push(var);
        self
    }

    pub fn env_from_secret<
        N: Into<String>,
        S: Into<String>,
        K: Into<String>
    >(&mut self, name: N, secret_name: S, secret_key: K) -> &mut Self {
        let var = EnvVar {
            name: name.into(),
            value_from: Some(EnvVarSource {
                secret_key_ref: Some(SecretKeySelector {
                    key: secret_key.into(),
                    name: secret_name.into(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        self.vars.push(var);
        self
    }

    pub fn build(&self) -> Vec<EnvVar> {
        self.vars.clone()
    }
}

pub fn http_probe<S: Into<String>>(path: S, port: IntOrString) -> Probe {
    Probe {
        failure_threshold: Some(3),
        http_get: Some(HTTPGetAction {
            path: Some(path.into()),
            port: port,
            scheme: Some("HTTP".into()),
            ..Default::default()
        }),
        period_seconds: Some(10),
        success_threshold: Some(1),
        timeout_seconds: Some(1),
        ..Default::default()
    }
}
