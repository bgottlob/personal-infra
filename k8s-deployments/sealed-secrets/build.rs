use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use serde_json::json;

const CHART_NAME: &str = "sealed-secrets";
const CHART_VERSION: &str = "2.18.5";
const NAMESPACE: &str = "sealed-secrets";
const REPO_NAME: &str = "sealed-secrets";
const REPO_URL: &str = "https://bitnami-labs.github.io/sealed-secrets";

fn main() -> anyhow::Result<()> {
    helm::ensure_repo(REPO_NAME, REPO_URL)?;

    let out_dir = env::var("OUT_DIR")?;
    let out_path = Path::new(&out_dir);

    helm::pull(Some(REPO_NAME), CHART_NAME, CHART_VERSION, out_path)?;

    let mut out_file = BufWriter::new(
        File::create(out_path.join("helm-output.yaml"))?
    );

    let template = helm::template(CHART_NAME, CHART_VERSION, NAMESPACE, helm::TemplateOptions {
        release_name: "sealed-secrets",
        include_crds: true,
        values: Some(json!({
            "resources": {
                "requests": { "cpu": "20m", "memory": "64Mi" },
                "limits":   { "cpu": "100m", "memory": "64Mi" },
            },
        })),
        ..Default::default()
    }, out_path)?;
    write!(&mut out_file, "{}", template)?;

    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
