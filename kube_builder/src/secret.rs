use anyhow::anyhow;
use std::collections::BTreeMap;
use base64::prelude::*;

use k8s_openapi::{api::core::v1::Secret, apimachinery::pkg::apis::meta::v1::ObjectMeta};

#[derive(Default)]
pub struct SecretBuilder {
    name: Option<String>,
    string_data: BTreeMap<String, String>,
}

impl SecretBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name<S: Into<String>>(&mut self, name: S) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    pub fn value<S: Into<String>>(&mut self, key: S, value: S) -> &mut Self {
        self.string_data.insert(key.into(), value.into());
        self
    }

    pub fn build(&self) -> anyhow::Result<Secret> {
        let name = self.name.clone().ok_or(anyhow!("Name must be set"))?;

        let string_data = match self.string_data.is_empty() {
            false => Ok(self.string_data.clone()),
            true => Err(anyhow!("At least one secret key/value pair must be added")),
        }?;

        Ok(Secret {
            metadata: ObjectMeta {
                name: Some(name),
                ..Default::default()
            },
            string_data: Some(string_data),
            ..Default::default()
        })
    }
}

pub fn docker_registry_secret(server: String, username: String, password: String, email: String) -> Secret {
    let mut string_data: BTreeMap<String, String> = BTreeMap::new();

    let value = k8s_openapi::serde_json::json!({
        "auths": {
            server: {
                "username": username,
                "password": password,
                "email": email,
                "auth": BASE64_STANDARD.encode(format!("{}:{}", username, password)),
            }
        }
    });

    string_data.insert(".dockerconfigjson".into(), value.to_string());

    Secret {
        metadata: ObjectMeta {
            name: Some("registry-creds".into()),
            ..Default::default()
        },
        type_: Some("kubernetes.io/dockerconfigjson".into()),
        string_data: Some(string_data),
        ..Default::default()
    }
}
