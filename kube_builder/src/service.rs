use anyhow::anyhow;
use crate::PortProtocol;
use std::collections::BTreeMap;

use k8s_openapi::{
    api::core::v1::{Service, ServicePort, ServiceSpec},
    apimachinery::pkg::apis::meta::v1::ObjectMeta,
};

#[derive(Default)]
pub struct ServiceBuilder {
    name: Option<String>,
    ports: Vec<ServicePort>,
    selector: BTreeMap<String, String>,
    annotations: BTreeMap<String, String>,
}

impl ServiceBuilder {
    pub fn new() -> Self {
        ServiceBuilder::default()
    }

    pub fn annotation<S: Into<String>, T: Into<String>>(&mut self, key: S, value: T) -> &mut Self {
        self.annotations.insert(key.into(), value.into());
        self
    }

    pub fn cluster_issuer_annotation<S: Into<String>>(
        &mut self,
        cluster_issuer: Option<S>,
    ) -> &mut Self {
        let ci_str: String = match cluster_issuer {
            Some(issuer) => issuer.into(),
            None => String::from("letsencrypt-prod"),
        };
        self.annotation("cert-manager.io/cluster-issuer", ci_str);
        self
    }

    pub fn selector_label<S: Into<String>, T: Into<String>>(
        &mut self,
        key: S,
        val: T,
    ) -> &mut Self {
        self.selector.insert(key.into(), val.into());
        self
    }

    pub fn selector(&mut self, mut selector: BTreeMap<String, String>) -> &mut Self {
        self.selector.append(&mut selector);
        self
    }

    pub fn name<S: Into<String>>(&mut self, name: S) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    pub fn port<S: Into<String>>(
        &mut self,
        name: S,
        protocol: PortProtocol,
        port: i32,
        target_port: i32,
    ) -> &mut Self {
        let service_port = ServicePort {
            name: Some(name.into()),
            protocol: Some(protocol.to_string()),
            port,
            target_port: Some(
                k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(target_port),
            ),

            ..Default::default()
        };

        self.ports.push(service_port);
        self
    }

    pub fn build(&self) -> anyhow::Result<Service> {
        let name = self.name.clone().ok_or(anyhow!("Name must be set"))?;
        let annotations = self.annotations.clone();

        let selector = match self.selector.is_empty() {
            false => Ok(self.selector.clone()),
            true => Err(anyhow!(
                "At least one key/value pair label must be added to the selector"
            )),
        }?;

        let ports = match self.ports.is_empty() {
            false => Ok(self.ports.clone()),
            true => Err(anyhow!("At least one service port must be set")),
        }?;

        let service = Service {
            metadata: ObjectMeta {
                name: Some(name),
                annotations: Some(annotations),
                ..Default::default()
            },
            spec: Some(ServiceSpec {
                selector: Some(selector),
                ports: Some(ports),
                ..Default::default()
            }),
            ..Default::default()
        };

        Ok(service)
    }
}
