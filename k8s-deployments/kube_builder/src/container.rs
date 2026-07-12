use anyhow::anyhow;
use k8s_openapi::{api::core::v1::{Container, ContainerPort, EnvVar, EnvVarSource, HTTPGetAction, Probe, ResourceRequirements, SecretKeySelector, SecurityContext, VolumeMount}, apimachinery::pkg::{api::resource::Quantity, util::intstr::IntOrString}};

use crate::PortProtocol;

#[derive(Default)]
pub struct ContainerBuilder {
    name: Option<String>,
    image: Option<String>,
    resources: Option<ResourceRequirements>,
    container_ports: Vec<ContainerPort>,
    env: Vec<EnvVar>,
    command: Vec<String>,
    args: Vec<String>,
    liveness_probe: Option<Probe>,
    readiness_probe: Option<Probe>,
    volume_mounts: Vec<VolumeMount>,
    security_context: Option<SecurityContext>,
}

pub fn http_probe<P: Into<String>>(path: P, port: IntOrString, failure_threshold: Option<i32>, period_seconds: Option<i32>, timeout_seconds: Option<i32>, success_threshold: Option<i32>) -> Probe {
    Probe {
        failure_threshold: failure_threshold.or(Some(3)),
        http_get: Some(HTTPGetAction {
            path: Some(path.into()),
            port: port,
            scheme: Some("HTTP".into()),
            ..Default::default()
        }),
        period_seconds: period_seconds.or(Some(10)),
        success_threshold: success_threshold.or(Some(1)),
        timeout_seconds: timeout_seconds.or(Some(5)),
        ..Default::default()
    }
}

enum Resource {
    Cpu,
    Memory
}

enum ResourceType {
    Limits,
    Requests,
}

fn update_resources(resources: Option<ResourceRequirements>, resource: Resource, rtype: ResourceType, quantity: Quantity) -> Option<ResourceRequirements> {
    let mut resources = resources.unwrap_or_default();

    let mut rmap = match rtype {
        ResourceType::Limits   => resources.limits.clone(),
        ResourceType::Requests => resources.requests.clone(),
    }.unwrap_or_default();

    let rkey = match resource {
        Resource::Cpu => String::from("cpu"),
        Resource::Memory => String::from("memory"),
    };

    rmap.insert(rkey, quantity);

    match rtype {
        ResourceType::Limits   => resources.limits = Some(rmap),
        ResourceType::Requests => resources.requests = Some(rmap),
    };
    
    Some(resources)
}

impl ContainerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn image<I: Into<String>>(&mut self, image: I) -> &mut Self {
        self.image = Some(image.into());
        self
    }

    pub fn container_port<I: Into<i32>, N: Into<String>>(&mut self, number: I, name: N, protocol: PortProtocol) -> &mut Self {
        self.container_ports.push(ContainerPort {
            container_port: number.into(),
            name: Some(name.into()),
            protocol: Some(protocol.to_string()),
            ..Default::default()
        });
        self
    }

    pub fn cpu_request(&mut self, quantity: Quantity) -> &mut Self {
        self.resources = update_resources(self.resources.clone(), Resource::Cpu, ResourceType::Requests, quantity);
        self
    }

    pub fn cpu_limit(&mut self, quantity: Quantity) -> &mut Self {
        self.resources = update_resources(self.resources.clone(), Resource::Cpu, ResourceType::Limits, quantity);
        self
    }

    pub fn name<N: Into<String>>(&mut self, name: N) -> &mut Self {
        self.name = Some(name.into());
        self
    }
    
    pub fn memory_request(&mut self, quantity: Quantity) -> &mut Self {
        self.resources = update_resources(self.resources.clone(), Resource::Memory, ResourceType::Requests, quantity);
        self
    }

    pub fn memory_limit(&mut self, quantity: Quantity) -> &mut Self {
        self.resources = update_resources(self.resources.clone(), Resource::Memory, ResourceType::Limits, quantity);
        self
    }

    pub fn env<N: Into<String>, V: Into<String>>(&mut self, name: N, value: V) -> &mut Self {
        let var = EnvVar {
            name: name.into(),
            value: Some(value.into()),
            ..Default::default()
        };
        self.env.push(var);
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
        self.env.push(var);
        self
    }

    pub fn command<V: Into<Vec<String>>>(&mut self, command: V) -> &mut Self {
        let command: Vec<String> = command.into();
        self.command.append(&mut command.clone());
        self
    }

    pub fn arg<S: Into<String>>(&mut self, arg: S) -> &mut Self {
        self.args.push(arg.into());
        self
    }

    pub fn args<V: Into<Vec<String>>>(&mut self, args: V) -> &mut Self {
        let args: Vec<String> = args.into();
        self.args.append(&mut args.clone());
        self
    }

    pub fn readiness_probe(&mut self, probe: Probe) -> &mut Self {
        self.readiness_probe = Some(probe);
        self
    }

    pub fn liveness_probe(&mut self, probe: Probe) -> &mut Self {
        self.liveness_probe = Some(probe);
        self
    }

    pub fn security_context(&mut self, security_context: SecurityContext) -> &mut Self {
        self.security_context = Some(security_context);
        self
    }

    pub fn volume_mount(&mut self, volume_mount: VolumeMount) -> &mut Self {
        self.volume_mounts.push(volume_mount);
        self
    }

    pub fn build(&self) -> anyhow::Result<Container> {
        let name = self.name.clone().ok_or(anyhow!("Container must have a name"))?;

        let container_ports = match self.container_ports.is_empty() {
            true => None,
            false => Some(self.container_ports.clone()),
        };

        let env = match self.env.is_empty() {
            true => None,
            false => Some(self.env.clone()),
        };

        let command = match self.command.is_empty() {
            true => None,
            false => Some(self.command.clone()),
        };

        let args = match self.args.is_empty() {
            true => None,
            false => Some(self.args.clone()),
        };

        let volume_mounts = match self.volume_mounts.is_empty() {
            true => None,
            false => Some(self.volume_mounts.clone()),
        };

        let container = Container {
            command,
            args,
            name,
            env,
            image: self.image.clone(),
            resources: self.resources.clone(),
            ports: container_ports,
            liveness_probe: self.liveness_probe.clone(),
            readiness_probe: self.readiness_probe.clone(),
            security_context: self.security_context.clone(),
            volume_mounts,
            ..Default::default()
        };

        Ok(container)
    }
}

#[cfg(test)]
mod test {
    use crate::container::*;
    use k8s_openapi::apimachinery::pkg::api::resource::Quantity;

    #[test]
    fn test_resources() {
        let mut builder = ContainerBuilder::new();
        builder.name("test");

        let container = builder.build().expect("building container should succeed");
        assert_eq!(container.resources, None);

        builder.cpu_limit(Quantity(String::from("100m")));
        let container = builder.build().expect("building container should succeed");
        assert_eq!(
            *container.resources.unwrap().limits.unwrap().get(&String::from("cpu")).unwrap(),
            Quantity(String::from("100m"))
        );
    }

    #[test]
    fn test_args() {
        let mut builder = ContainerBuilder::new();
        builder.args(["test".to_string()]);
    }
}
