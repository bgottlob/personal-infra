use k8s_openapi::api::core::v1::Secret;
use serde_norway::Value;

use crate::input::{SecretTemplate, StringDataValue};
use std::collections::BTreeMap;

fn transform(template: SecretTemplate, secrets: &serde_norway::Value) -> anyhow::Result<BTreeMap<String, String>> {
    let mut acc: BTreeMap<String, String> = BTreeMap::new();
    for (key, value) in template.string_data.iter() {
        let secret_value = match value {
            StringDataValue::Literal(value) => value.clone(),
            StringDataValue::SopsRef { sops } => {
                get(secrets, sops).ok_or(anyhow::format_err!("Secret at path `{sops}` does not exist"))?
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
        string_data: Some(transform(template, secrets)?),
        ..Default::default()
    };
    Ok(secret)
}
