use std::{path::PathBuf, process::Command};

pub fn decrypt(secrets: PathBuf) -> anyhow::Result<serde_norway::Value> {
    let output = Command::new("sops")
        .arg("decrypt")
        .arg(&secrets)
        .output()?;
    if !output.status.success() {
        anyhow::bail!("sops decrypt failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    let stdout = str::from_utf8(&output.stdout)?;
    let value = serde_norway::from_str(stdout)?;
    Ok(value)
}
