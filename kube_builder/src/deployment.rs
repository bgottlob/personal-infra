use std::collections::BTreeMap;
use anyhow::anyhow;

use k8s_openapi::{api::{apps::v1::{Deployment, DeploymentSpec}, core::v1::{Container, ContainerPort, EnvVar, EnvVarSource, PodSpec, PodTemplateSpec, SecretKeySelector}}, apimachinery::pkg::apis::meta::v1::{LabelSelector, ObjectMeta}};

use crate::PortProtocol;

#[derive(Default)]
pub struct DeploymentBuilder {
    name: Option<String>,
    replicas: Option<i32>,
    selector_match_labels: BTreeMap<String, String>,
    pod_labels: BTreeMap<String, String>,
    containers: Vec<Container>,
}

impl DeploymentBuilder {
    pub fn new() -> Self {
        DeploymentBuilder::default()
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
    >(&mut self, name: N, image: I, port_name: P, port: i32, port_protocol: PortProtocol, env: Vec<EnvVar>) -> &mut Self {
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
            ..Default::default()
        };
        self.containers.push(container);
        self
    }

    pub fn build(&self) -> anyhow::Result<Deployment> {
        let name = self.name.clone().ok_or(anyhow!("Deployment must have a name"))?;

        let containers = match self.containers.is_empty() {
            false => Ok(self.containers.clone()),
            true => Err(anyhow!("At least one container must be added")),
        }?;

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
