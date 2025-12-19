use std::{collections::HashMap, process::Command};
use std::path::Path;
use anyhow::{anyhow, format_err};
use serde_json::Value;

pub fn repo_exists(name: &str, url: &str) -> bool {
    let helm_repo_list = Command::new("helm")
        .arg("repo")
        .arg("list")
        .args(["-o", "json"])
        .output()
        .expect("helm repo list should succeed")
        .stdout;
    let repos_str = str::from_utf8(&helm_repo_list).expect("helm repo list stdout output should convert to str");
    let repos_value: Value = serde_json::from_str(repos_str).expect("json from helm repo list should be parsed");

    match repos_value {
        Value::Array(repos) => {
            repos.iter().find(|elem| {
                if let Value::Object(obj) = elem {
                   obj.get(&String::from("name")) == Some(&Value::String(String::from(name)))
                       && obj.get(&String::from("url")) == Some(&Value::String(String::from(url)))
                } else {
                    false
                }
            })
            .is_some()
        },
        _ => panic!("json returned from helm repo list should be an array")
    }
}

pub fn add_repo(name: &str, url: &str) {
    let status = Command::new("helm")
        .arg("repo")
        .arg("add")
        .args([name, url])
        .status()
        .expect("helm repo add failed to complete");
    if !status.success() {
        panic!("helm repo add failed with exit code {}", status);
    }
}

pub fn pull(repo_name: &str, chart_name: &str, chart_version: &str, destination: &Path) -> anyhow::Result<()> {
    let destination_str = destination.to_str().ok_or(format_err!("destination path should convert to str"))?;
    let status = Command::new("helm")
        .arg("pull")
        .arg(format!("{}/{}", repo_name, chart_name))
        .args(["--version", chart_version])
        .args(["--destination", destination_str])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("helm pull failed with exit code {}", status))
    }

}

pub fn template(chart_name: &str, chart_version: &str, namespace: &str, release_name: Option<&str>, set_values: Option<HashMap<&str, &str>>, values_file: Option<&Path>, source_dir: &Path) -> anyhow::Result<String> {
    let path = source_dir.join(format!("{}-{}.tgz", chart_name, chart_version));
    let mut cmd = Command::new("helm");
    cmd.arg("template");

    if let Some(release_name) = release_name {
        cmd.args(["--release-name", release_name]);
    }

    cmd.args(["--namespace", namespace]);
    cmd.arg(path.to_str().unwrap());

    set_values.unwrap_or_default().iter().for_each(|(key, val)| {
        cmd.args(["--set", format!("{}={}", key, val).as_str()]);
    });

    if let Some(values_path) = values_file {
        cmd.args([
            "--values",
            source_dir.join(values_path).to_str().expect("conversion of values file path to str should succeed")
        ]);
    };

    let output = cmd.output()?;

    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)?;
        Ok(stdout)
    } else {
        let stderr = String::from_utf8(output.stderr)?;
        Err(anyhow!("helm template failed with exit code {}: `{}`", output.status, stderr))
    }
}
