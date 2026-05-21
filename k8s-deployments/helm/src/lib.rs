use std::fs;
use std::{collections::HashMap, process::{Command, ExitStatus}};
use std::path::Path;
use serde_json::Value;

/// Errors that can occur when invoking helm commands.
#[derive(Debug, thiserror::Error)]
pub enum HelmError {
    /// A helm subcommand exited with a non-zero status code.
    #[error("helm {subcommand} failed (exit {status}): {stderr}")]
    CommandFailed {
        subcommand: &'static str,
        status: ExitStatus,
        stderr: String,
    },

    /// `helm repo list` returned JSON that was not an array of repo objects.
    #[error("helm repo list returned unexpected JSON format (expected array)")]
    UnexpectedRepoListFormat,

    /// No `.tgz` file matching the expected chart name and version was found in
    /// the source directory after `helm pull`.
    #[error("no chart file found matching {chart}-{version}.tgz in {dir}")]
    ChartNotFound {
        chart: String,
        version: String,
        dir: String,
    },

    /// A filesystem path could not be converted to a UTF-8 string.
    #[error("path contains non-UTF-8 characters")]
    NonUtf8Path,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),
}

pub type Result<T> = std::result::Result<T, HelmError>;

/// Options for [`template`]. Passed as a single struct to avoid a long
/// positional argument list.
pub struct TemplateOptions<'a> {
    /// The helm release name (`--release-name`).
    pub release_name: &'a str,
    /// Key-value pairs passed as `--set key=value` flags. Use an empty
    /// `HashMap` when no `--set` overrides are needed.
    pub set_values: HashMap<&'a str, &'a str>,
    /// A JSON value serialized to a temporary file and passed via `--values`.
    /// Use this for complex or nested chart configuration. Mutually exclusive
    /// with `set_values` in practice — helm accepts both, but callers
    /// typically use one or the other.
    pub values: Option<Value>,
    /// Include CRDs in the template output (`--include-crds`). Defaults to
    /// `false`; set to `true` for charts that install CRDs.
    pub include_crds: bool,
}

impl Default for TemplateOptions<'_> {
    fn default() -> Self {
        Self {
            release_name: "",
            set_values: HashMap::new(),
            values: None,
            include_crds: false,
        }
    }
}

/// Returns `true` if a repo with the given `name` and `url` is already
/// registered in the local helm repo list.
///
/// Returns `false` (rather than an error) when no repos are configured at all,
/// since helm exits non-zero in that case.
pub fn repo_exists(name: &str, url: &str) -> Result<bool> {
    let output = Command::new("helm")
        .args(["repo", "list", "-o", "json"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // helm repo list exits non-zero with this message when no repos are configured
        if stderr.contains("no repositories to show") {
            return Ok(false);
        }
        return Err(HelmError::CommandFailed {
            subcommand: "repo list",
            status: output.status,
            stderr: stderr.into_owned(),
        });
    }

    let stdout = String::from_utf8(output.stdout)?;
    let repos: Value = serde_json::from_str(&stdout)?;

    match repos {
        Value::Array(repos) => Ok(repos.iter().any(|elem| {
            if let Value::Object(obj) = elem {
                obj.get("name").and_then(Value::as_str) == Some(name)
                    && obj.get("url").and_then(Value::as_str) == Some(url)
            } else {
                false
            }
        })),
        _ => Err(HelmError::UnexpectedRepoListFormat),
    }
}

/// Registers a helm repo with the given `name` and `url` (`helm repo add`).
pub fn add_repo(name: &str, url: &str) -> Result<()> {
    let output = Command::new("helm")
        .args(["repo", "add", name, url])
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(HelmError::CommandFailed {
            subcommand: "repo add",
            status: output.status,
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

/// Ensures a helm repo with the given `name` and `url` is registered,
/// adding it via [`add_repo`] if it is not already present.
pub fn ensure_repo(name: &str, url: &str) -> Result<()> {
    if !repo_exists(name, url)? {
        add_repo(name, url)?;
    }
    Ok(())
}

/// Downloads a chart archive to `destination` (`helm pull`).
///
/// `repo_name` is the locally registered repo name. Pass `None` for OCI or
/// fully-qualified chart references where no repo prefix is needed.
pub fn pull(repo_name: Option<&str>, chart_name: &str, chart_version: &str, destination: &Path) -> Result<()> {
    let destination_str = destination.to_str().ok_or(HelmError::NonUtf8Path)?;

    let mut cmd = Command::new("helm");
    cmd.arg("pull");

    if let Some(repo_name) = repo_name {
        cmd.arg(format!("{}/{}", repo_name, chart_name));
    } else {
        cmd.arg(chart_name);
    }

    cmd.args(["--version", chart_version]);
    cmd.args(["--destination", destination_str]);

    let output = cmd.output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(HelmError::CommandFailed {
            subcommand: "pull",
            status: output.status,
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

/// Renders a chart's templates to a YAML string (`helm template`).
///
/// For non-OCI charts, the chart archive must already be present in
/// `source_dir` (i.e. [`pull`] must have been called first). The archive is
/// located by searching `source_dir` for a `.tgz` file whose name starts with
/// `{chart_name}-{chart_version}`.
///
/// For OCI chart references (`oci://...`), the chart URL is passed directly to
/// helm without a local file lookup.
pub fn template(chart_name: &str, chart_version: &str, namespace: &str, options: TemplateOptions<'_>, source_dir: &Path) -> Result<String> {
    let mut cmd = Command::new("helm");
    cmd.arg("template");

    cmd.args(["--release-name", options.release_name]);
    cmd.args(["--namespace", namespace]);

    if chart_name.starts_with("oci://") {
        cmd.arg(chart_name);
        cmd.args(["--version", chart_version]);
    } else {
        // Locate the pulled chart archive. Some repos append a digest suffix
        // after the version, so we match on prefix + ".tgz" rather than an
        // exact filename.
        let chart_prefix = format!("{}-{}", chart_name, chart_version);
        let found = fs::read_dir(source_dir)?
            .find_map(|entry| {
                let entry = entry.ok()?;
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with(&chart_prefix) && name.ends_with(".tgz") {
                    Some(entry.path())
                } else {
                    None
                }
            })
            .ok_or_else(|| HelmError::ChartNotFound {
                chart: chart_name.to_string(),
                version: chart_version.to_string(),
                dir: source_dir.display().to_string(),
            })?;

        cmd.arg(found);
    }

    if options.include_crds {
        cmd.arg("--include-crds");
    }

    for (key, val) in &options.set_values {
        cmd.args(["--set", &format!("{}={}", key, val)]);
    }

    if let Some(values) = options.values {
        // helm accepts JSON as a values file since JSON is valid YAML
        let safe_name = chart_name.split('/').last().unwrap_or(chart_name);
        let values_path = source_dir.join(format!("{}-values.json", safe_name));
        fs::write(&values_path, serde_json::to_string_pretty(&values)?)?;
        cmd.args([
            "--values",
            values_path.to_str().ok_or(HelmError::NonUtf8Path)?,
        ]);
    }

    let output = cmd.output()?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        Err(HelmError::CommandFailed {
            subcommand: "template",
            status: output.status,
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}
