use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::{fs::File, io::Read, path::PathBuf};

#[derive(Deserialize, Debug)]
pub struct SecretTemplate {
    pub metadata: ObjectMeta,
    #[serde(rename = "stringData", default)]
    pub string_data: BTreeMap<String, StringDataValue>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum StringDataValue {
    Literal(String),
    SopsRef { sops: String },
}

pub fn read(template: PathBuf) -> anyhow::Result<Vec<SecretTemplate>> {
    let mut buffer = String::new();
    File::open(template)?.read_to_string(&mut buffer)?;

    let mut documents = Vec::new();
    for document in serde_norway::Deserializer::from_str(&buffer) {
        let secret: SecretTemplate = SecretTemplate::deserialize(document)?;
        documents.push(secret);
    }

    Ok(documents)
}
