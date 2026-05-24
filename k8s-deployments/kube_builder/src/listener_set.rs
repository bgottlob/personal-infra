use anyhow::anyhow;
use k8s_gateway_api::prelude::*;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct ListenerSetBuilder {
    name: Option<String>,
    hostname: Option<String>,
}

impl ListenerSetBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name<N: Into<String>>(&mut self, name: N) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    pub fn hostname<H: Into<String>>(&mut self, hostname: H) -> &mut Self {
        self.hostname = Some(hostname.into());
        self
    }

    pub fn build(&self) -> anyhow::Result<ListenerSet> {
        let name = self.name.clone().ok_or(anyhow!("ListenerSet must have a name"))?;
        let hostname = self.hostname.clone().ok_or(anyhow!("ListenerSet must have a hostname"))?;

        Ok(ListenerSet {
            metadata: ObjectMeta {
                name: Some(name.clone()),
                annotations: Some(BTreeMap::from([
                    ("cert-manager.io/cluster-issuer".to_string(), "letsencrypt-prod".to_string()),
                ])),
                ..Default::default()
            },
            spec: ListenerSetSpec {
                parent_ref: ListenerSetParentRef {
                    name: "envoy-public".to_string(),
                    namespace: Some("envoy-gateway-system".to_string()),
                    ..Default::default()
                },
                listeners: vec![ListenerSetListeners {
                    name: format!("{}-https", name),
                    protocol: "HTTPS".to_string(),
                    port: 443,
                    hostname: Some(hostname),
                    tls: Some(ListenerSetListenersTls {
                        mode: Some(ListenerSetListenersTlsMode::Terminate),
                        certificate_refs: Some(vec![ListenerSetListenersTlsCertificateRefs {
                            name: format!("{}-https", name),
                            kind: Some("Secret".to_string()),
                            ..Default::default()
                        }]),
                        ..Default::default()
                    }),
                    allowed_routes: Some(ListenerSetListenersAllowedRoutes {
                        namespaces: Some(ListenerSetListenersAllowedRoutesNamespaces {
                            from: Some(ListenerSetListenersAllowedRoutesNamespacesFrom::Same),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
            },
            ..Default::default()
        })
    }
}
