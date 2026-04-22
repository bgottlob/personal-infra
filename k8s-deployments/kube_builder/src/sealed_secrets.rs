use sealed_secrets_crd::sealed_secrets::SealedSecret;
use serde::Deserialize;
use std::io::Read;

#[derive(Deserialize)]
struct Input {
    #[serde(rename = "sealed-secrets")]
    sealed_secrets: Option<Vec<SealedSecret>>,
}

pub fn read_sealed_secrets_from_stdin() -> anyhow::Result<Vec<serde_json::Value>> {
    let mut stdin = String::new();
    std::io::stdin().read_to_string(&mut stdin)?;

    if stdin.trim().is_empty() {
        return Ok(vec![]);
    }

    let input: Input = serde_json::from_str(&stdin)?;
    input
        .sealed_secrets
        .unwrap_or_default()
        .into_iter()
        .map(|s| Ok(serde_json::to_value(s)?))
        .collect()
}
