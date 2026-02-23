use std::fs;
use std::{collections::HashMap, process::Command};
use std::path::Path;
use anyhow::{anyhow, format_err};
use regex::Regex;
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

pub fn pull(repo_name: Option<&str>, chart_name: &str, chart_version: &str, destination: &Path) -> anyhow::Result<()> {
    let destination_str = destination.to_str().ok_or(format_err!("destination path should convert to str"))?;
    let mut cmd = Command::new("helm");
    cmd.arg("pull");

    if let Some(repo_name) = repo_name {
        cmd.arg(format!("{}/{}", repo_name, chart_name));
    } else {
        cmd.arg(chart_name);
    }

    cmd.args(["--version", chart_version]);
    cmd.args(["--destination", destination_str]);
    let status = cmd.status()?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("helm pull failed with exit code {}", status))
    }

}

pub fn template(chart_name: &str, chart_version: &str, namespace: &str, release_name: Option<&str>, set_values: Option<HashMap<&str, &str>>, values_file: Option<&Path>, source_dir: &Path) -> anyhow::Result<String> {
    let mut cmd = Command::new("helm");
    cmd.arg("template");

    if let Some(release_name) = release_name {
        cmd.args(["--release-name", release_name]);
    }

    cmd.args(["--namespace", namespace]);
    if chart_name.starts_with("oci://") {
        cmd.arg(chart_name);
    } else {
        // Find the pulled helm chart with the given version; some repos add
        // the digest to the end of the filename
        let mut files = fs::read_dir(source_dir)
            .expect("should be able to read source directory {}");
        let chart_prefix = format!("{}-{}", chart_name, chart_version);
        let escaped = regex::escape(&chart_prefix);
        let pattern = format!(r"/{}([-\w]+)?.tgz$", escaped);
        let re = Regex::new(&pattern).expect("regex should be created from chart name and version");

        let found = files
            .find_map(|filename| {
                let filename_str = filename.expect("filename should be able to be decoded").path().display().to_string();

                if re.is_match(&filename_str) {
                    Some(filename_str)
                } else {
                    None
                }
            });

        // TODO fix unwrap
        let chart_path = source_dir.join(found.expect("unable to find helm chart file"));
        cmd.arg(chart_path);
    }

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
