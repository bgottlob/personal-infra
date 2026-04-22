use base64::prelude::*;
use k8s_openapi::api::core::v1::Secret;
use serde_norway::Value;

use crate::input::{SecretTemplate, StringDataValue};
use std::collections::BTreeMap;

fn transform(template: &SecretTemplate, secrets: &serde_norway::Value) -> anyhow::Result<BTreeMap<String, String>> {
    let mut acc: BTreeMap<String, String> = BTreeMap::new();
    for (key, value) in template.string_data.iter() {
        let secret_value = match value {
            StringDataValue::Literal(value) => value.clone(),
            StringDataValue::SopsRef { sops } => {
                get(secrets, sops).ok_or(anyhow::format_err!("Secret at path `{sops}` does not exist"))?
            }
            StringDataValue::SopsDockerRegistry { sops_docker_registry: r } => {
                let server   = get(secrets, &r.server).ok_or(anyhow::format_err!("Secret at path `{}` does not exist", r.server))?;
                let username = get(secrets, &r.username).ok_or(anyhow::format_err!("Secret at path `{}` does not exist", r.username))?;
                let password = get(secrets, &r.password).ok_or(anyhow::format_err!("Secret at path `{}` does not exist", r.password))?;
                let email    = get(secrets, &r.email).ok_or(anyhow::format_err!("Secret at path `{}` does not exist", r.email))?;
                let auth     = BASE64_STANDARD.encode(format!("{}:{}", username, password));
                serde_json::json!({
                    "auths": {
                        server: {
                            "username": username,
                            "password": password,
                            "email": email,
                            "auth": auth,
                        }
                    }
                }).to_string()
            }
        };
        acc.insert(key.to_string(), secret_value);
    }
    Ok(acc)
}

fn get(root: &serde_norway::Value, path: &str) -> Option<String> {
    let mut curr = root;
    for key in path.split('.') {
        curr = curr.get(key)?;
    }
    match curr {
        Value::String(value) => Some(value.to_string()),
        _ => None
    }
}

pub fn build_secret(template: SecretTemplate, secrets: &serde_norway::Value) -> anyhow::Result<Secret> {
    let secret = Secret {
        metadata: template.metadata.clone(),
        type_: template.type_.clone(),
        string_data: Some(transform(&template, secrets)?),
        ..Default::default()
    };
    Ok(secret)
}
