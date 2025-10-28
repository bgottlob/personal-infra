use std::process::Command;
use serde::Deserialize;

#[derive(Deserialize, PartialEq, Debug)]
pub struct Secrets {
    pub postgres: PostgresSecrets,
    pub miniflux: MinifluxSecrets,
    pub tailscale: TailscaleSecrets,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct PostgresSecrets {
    pub app_user: UserCredentials,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct MinifluxSecrets {
    pub admin: UserCredentials,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct TailscaleSecrets {
    pub authkey: String,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct UserCredentials {
    pub username: String,
    pub password: String,
}

pub fn decrypt_parse_secrets(path: &str) -> anyhow::Result<Secrets> {
    let output = Command::new("sops")
        .arg("decrypt")
        .arg(&path)
        .output()?;
    // TODO check the status of the sops command
    let stdout = str::from_utf8(&output.stdout)?;
    let all_secrets: Secrets = serde_norway::from_str(stdout)?;
    Ok(all_secrets)
}
