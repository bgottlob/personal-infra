use anyhow::anyhow;
use std::collections::BTreeMap;

use k8s_openapi::{api::networking::v1::{
    HTTPIngressPath, HTTPIngressRuleValue, Ingress, IngressBackend, IngressRule, IngressServiceBackend, IngressSpec, IngressTLS, ServiceBackendPort
}, apimachinery::pkg::apis::meta::v1::ObjectMeta};

#[derive(Default)]
pub struct IngressBuilder {
    name: Option<String>,
    annotations: BTreeMap<String, String>,
    ingress_class_name: Option<String>,
    tls_hosts: Vec<IngressTLS>,
    rules: Vec<IngressRule>,
}

impl IngressBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name<S: Into<String>>(&mut self, name: S) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    pub fn annotation<S: Into<String>, T: Into<String>>(&mut self, key: S, value: T) -> &mut Self {
        self.annotations.insert(key.into(), value.into());
        self
    }

    pub fn ingress_class_name<S: Into<String>>(&mut self, name: S) -> &mut Self {
        self.ingress_class_name = Some(name.into());
        self
    }

    pub fn tls_host<S: Into<String>, T: Into<String>>(
        &mut self,
        hostname: S,
        secret_prefix: T,
    ) -> &mut Self {
        let value = IngressTLS {
            hosts: Some(vec![hostname.into()]),
            secret_name: Some(format!("{}-tls", secret_prefix.into())),
        };
        self.tls_hosts.push(value);
        self
    }

    pub fn tls_hosts<S: Into<String>>(
        &mut self,
        hostnames: Vec<String>,
        secret_prefix: S,
    ) -> &mut Self {
        let value = IngressTLS {
            hosts: Some(hostnames),
            secret_name: Some(format!("{}-tls", secret_prefix.into())),
        };
        self.tls_hosts.push(value);
        self
    }

    // TODO have a solution for multiple paths in one ingress rule
    // TODO enum for path_type
    pub fn rule<H: Into<String>, P: Into<String>, T: Into<String>, N: Into<String>>(
        &mut self,
        hostname: H,
        path: P,
        path_type: T,
        service_name: N,
        service_port: i32,
    ) -> &mut Self {
        let rule = IngressRule {
            host: Some(hostname.into()),
            http: Some(HTTPIngressRuleValue {
                paths: vec![HTTPIngressPath {
                    path_type: path_type.into(),
                    path: Some(path.into()),
                    backend: IngressBackend {
                        service: Some(IngressServiceBackend {
                            name: service_name.into(),
                            port: Some(ServiceBackendPort {
                                number: Some(service_port),
                                ..Default::default()
                            }),
                        }),
                        ..Default::default()
                    },
                }],
            }),
        };

        self.rules.push(rule);
        self
    }

    pub fn build(&self) -> anyhow::Result<Ingress> {
        let name = self.name.clone().ok_or(anyhow!("The ingress must have a name"))?;
        let ingress_class_name = self.ingress_class_name.clone().ok_or(anyhow!("The ingress must have an ingress class name set"))?;
        let rules = match self.rules.is_empty() {
            false => Ok(self.rules.clone()),
            true => Err(anyhow!("There must be at least one ingress rule set"))
        }?;

        let annotations = if self.annotations.is_empty() {
            None
        } else {
            Some(self.annotations.clone())
        };

        let tls = if self.tls_hosts.is_empty() {
            None
        } else {
            Some(self.tls_hosts.clone())
        };

        let ingress = Ingress {
            metadata: ObjectMeta {
                name: Some(name),
                annotations,
                ..Default::default()
            },
            spec: Some(IngressSpec {
                ingress_class_name: Some(ingress_class_name),
                tls,
                rules: Some(rules),
                ..Default::default()
            }),
            ..Default::default()
        };

        Ok(ingress)
    }
}
