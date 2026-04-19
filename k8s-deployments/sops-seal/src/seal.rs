use std::{io::Write, process::{Command, Stdio}};

pub fn seal_token(token: String, format: &str) -> anyhow::Result<String> {
    let mut cmd = Command::new("kubeseal")
        .arg("--controller-name=sealed-secrets")
        .arg("--controller-namespace=sealed-secrets")
        .args(["--format", format])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    cmd
        .stdin
        .take()
        .ok_or(anyhow::format_err!("Unable to pipe stdin"))?
        .write_all(token.as_bytes())?;

    let output = cmd.wait_with_output()?;

    if !output.status.success() {
        anyhow::bail!("sealing a secret failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(String::from_utf8(output.stdout)?)
}
