use std::process::Command;
use serde::Deserialize;

#[derive(Deserialize, PartialEq, Debug)]
pub struct Secrets {
    pub postgres: PostgresSecrets,
    pub miniflux: MinifluxSecrets,
    pub tailscale: TailscaleSecrets,
    pub registry: RegistrySecrets,
    pub linode: LinodeSecrets,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct RegistrySecrets {
    pub auth: RegistryAuth,
    pub bucket: CloudKeyPair,
    pub login: RegistryLogin,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct RegistryAuth {
    pub htpasswd: String,
}

// An access and secret key pair used to authenticate to public cloud services
#[derive(Deserialize, PartialEq, Debug)]
pub struct CloudKeyPair {
    pub access_key_id: String,
    pub secret_key: String,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct RegistryLogin {
    pub server: String,
    pub username: String,
    pub password: String,
    pub email: String,
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

#[derive(Deserialize, PartialEq, Debug)]
pub struct LinodeSecrets {
    pub csi_driver_token: String,
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
