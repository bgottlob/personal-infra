use anyhow::anyhow;
use k8s_gateway_api::prelude::*;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

#[derive(Default)]
pub struct HTTPRouteBuilder {
    name: Option<String>,
    parent_refs: Vec<HttpRouteParentRefs>,
    hostnames: Vec<String>,
    rules: Vec<HttpRouteRules>
}

impl HTTPRouteBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name<N: Into<String>>(&mut self, name: N) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    pub fn hostname<N: Into<String>>(&mut self, hostname: N) -> &mut Self {
        self.hostnames.push(hostname.into());
        self
    }

    pub fn gateway_parent_ref<S: Into<String>, N: Into<String>>(&mut self, namespace: S, name: N) -> &mut Self {
        let parent_ref = HttpRouteParentRefs {
            namespace: Some(namespace.into()),
            name: name.into(),
            kind: Some(String::from("Gateway")),
            ..Default::default()
        };
        self.parent_refs.push(parent_ref);
        self
    }

    // Adds a parentRef targeting a specific listener on a ListenerSet, identified
    // by its section name. Required for HTTPS routes when the listener lives on a
    // ListenerSet rather than directly on the Gateway.
    pub fn listener_set_parent_ref<N: Into<String>, S: Into<String>>(&mut self, name: N, section_name: S) -> &mut Self {
        let parent_ref = HttpRouteParentRefs {
            name: name.into(),
            kind: Some(String::from("ListenerSet")),
            group: Some(String::from("gateway.networking.k8s.io")),
            section_name: Some(section_name.into()),
            ..Default::default()
        };
        self.parent_refs.push(parent_ref);
        self
    }

    pub fn parent_ref(&mut self, parent_ref: HttpRouteParentRefs) -> &mut Self {
        self.parent_refs.push(parent_ref);
        self
    }

    pub fn https_redirect_rule(&mut self) -> &mut Self {
        let rule = HttpRouteRules {
            filters: Some(vec![HttpRouteRulesFilters {
                r#type: HttpRouteRulesFiltersType::RequestRedirect,
                request_redirect: Some(HttpRouteRulesFiltersRequestRedirect {
                    scheme: Some(HttpRouteRulesFiltersRequestRedirectScheme::Https),
                    status_code: Some(301),
                    ..Default::default()
                }),
                cors: None,
                extension_ref: None,
                external_auth: None,
                request_header_modifier: None,
                request_mirror: None,
                response_header_modifier: None,
                url_rewrite: None,
            }]),
            ..Default::default()
        };
        self.rules.push(rule);
        self
    }

    pub fn service_port_backend_rule<N: Into<String>>(&mut self, name: N, port: i32) -> &mut Self {
        let rule = HttpRouteRules {
            backend_refs: Some(vec![
               HttpRouteRulesBackendRefs {
                   name: name.into(),
                   port: Some(port),
                   ..Default::default()
               }
            ]),
            ..Default::default()
        };
        self.rules.push(rule);
        self
    }

    pub fn rule<N: Into<String>>(&mut self, rule: HttpRouteRules) -> &mut Self {
        self.rules.push(rule);
        self
    }

    pub fn build(&self) -> anyhow::Result<HTTPRoute> {
        let name = self.name.clone().ok_or(anyhow!("The http route must have a name"))?;

        let hostnames = if self.hostnames.is_empty() {
            None
        } else {
            Some(self.hostnames.clone())
        };

        let rules = if self.rules.is_empty() {
            None
        } else {
            Some(self.rules.clone())
        };

        let parent_refs = if self.parent_refs.is_empty() {
            None
        } else {
            Some(self.parent_refs.clone())
        };

        let http_route = HTTPRoute {
            metadata: ObjectMeta {
                name: Some(name),
                ..Default::default()
            },
            spec: HttpRouteSpec {
                hostnames,
                rules,
                parent_refs,
                ..Default::default()
            },
            ..Default::default()
        };

        Ok(http_route)
    }
}
