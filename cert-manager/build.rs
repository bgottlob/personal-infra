use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

const CHART_VERSION: &str = "v1.12.17";
const REPO_NAME: &str = "jetstack";
const REPO_URL: &str = "https://charts.jetstack.io";
const CHART_NAME: &str = "cert-manager";
const NAMESPACE: &str = "cert-manager";

fn main() -> anyhow::Result<()> {
    if !helm::repo_exists(REPO_NAME, REPO_URL) {
        helm::add_repo(REPO_NAME, REPO_URL)
    }
    let out_dir = env::var("OUT_DIR")?;
    let out_path = Path::new(&out_dir);

    helm::pull(REPO_NAME, CHART_NAME, CHART_VERSION, out_path)?;

    let mut file = BufWriter::new(
        File::create(out_path.join("helm-output.yaml"))?
    );

    let values = HashMap::from([
        ("crds.enabled", "true")
    ]);
    let template = helm::template(CHART_NAME, CHART_VERSION, NAMESPACE, Some(CHART_NAME), Some(values), None, out_path)?;
    write!(&mut file, "{}", template)?;
    Ok(())
}

