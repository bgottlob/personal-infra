use std::env;
use serde::Deserialize;

fn main() -> anyhow::Result<()> {
    let cnpg_out_str = include_str!(concat!(env!("OUT_DIR"), "/cnpg-helm-output.yaml"));
    let barman_out_str = include_str!(concat!(env!("OUT_DIR"), "/barman-helm-output.yaml"));
    let mut resources: Vec<serde_json::Value> = Vec::new();
    for document in serde_norway::Deserializer::from_str(cnpg_out_str) {
        let value = serde_norway::Value::deserialize(document)?;
        if value != serde_norway::Value::Null {
            resources.push(serde_json::to_value(value)?);
        }
    }
    for document in serde_norway::Deserializer::from_str(barman_out_str) {
        let value = serde_norway::Value::deserialize(document)?;
        if value != serde_norway::Value::Null {
            resources.push(serde_json::to_value(value)?);
        }
    }
    println!("{}", serde_json::to_string(&resources).unwrap());
    Ok(())
}
