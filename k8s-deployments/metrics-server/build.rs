use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

const CHART_VERSION: &str = "3.13.0";
const REPO_NAME: &str = "metrics-server";
const REPO_URL: &str = "https://kubernetes-sigs.github.io/metrics-server/";
const CHART_NAME: &str = "metrics-server";
const NAMESPACE: &str = "kube-system";

fn main() -> anyhow::Result<()> {
    helm::ensure_repo(REPO_NAME, REPO_URL)?;
    let out_dir = env::var("OUT_DIR")?;
    let out_path = Path::new(&out_dir);

    helm::pull(Some(REPO_NAME), CHART_NAME, CHART_VERSION, out_path)?;

    let mut file = BufWriter::new(
        File::create(out_path.join("helm-output.yaml"))?
    );

    let template = helm::template(CHART_NAME, CHART_VERSION, NAMESPACE, helm::TemplateOptions {
        release_name: CHART_NAME,
        set_values: HashMap::new(),
        ..Default::default()
    }, out_path)?;
    write!(&mut file, "{}", template)?;
    Ok(())
}
