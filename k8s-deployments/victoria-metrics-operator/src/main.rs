use std::env;
use serde::Deserialize;

fn main() -> anyhow::Result<()> {
    let helm_out_str = include_str!(concat!(env!("OUT_DIR"), "/helm-output.yaml"));
    let mut resources: Vec<serde_json::Value> = Vec::new();
    for document in serde_norway::Deserializer::from_str(helm_out_str) {
        let value = serde_norway::Value::deserialize(document)?;
        if value != serde_norway::Value::Null {
            let mut resource = serde_json::to_value(value)?;

            // The operator manages its own TLS cert rotation via the vmo-validation
            // Secret. Letting yoke own it causes a perpetual diff as the operator
            // continuously rotates the certs back after each takeoff.
            if is_vmo_validation_secret(&resource) {
                continue;
            }

            // For the same reason, clear caBundle from all webhook configurations.
            // The operator injects the correct CA after cert rotation; yoke should
            // not fight it by re-applying the stale build-time cert.
            clear_ca_bundles(&mut resource);

            resources.push(resource);
        }
    }
    println!("{}", serde_json::to_string(&resources).unwrap());
    Ok(())
}

fn is_vmo_validation_secret(resource: &serde_json::Value) -> bool {
    resource.get("kind").and_then(|v| v.as_str()) == Some("Secret")
        && resource
            .pointer("/metadata/name")
            .and_then(|v| v.as_str())
            == Some("vmo-validation")
}

fn clear_ca_bundles(resource: &mut serde_json::Value) {
    let kind = resource
        .get("kind")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let webhook_key = match kind {
        "MutatingWebhookConfiguration" => "webhooks",
        "ValidatingWebhookConfiguration" => "webhooks",
        _ => return,
    };
    if let Some(webhooks) = resource
        .get_mut(webhook_key)
        .and_then(|v| v.as_array_mut())
    {
        for webhook in webhooks {
            if let Some(client_config) = webhook.get_mut("clientConfig") {
                if let Some(obj) = client_config.as_object_mut() {
                    obj.remove("caBundle");
                }
            }
        }
    }
}
