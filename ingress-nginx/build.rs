use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

const CHART_VERSION: &str = "4.12.1";
const REPO_NAME: &str = "ingress-nginx";
const REPO_URL: &str = "https://kubernetes.github.io/ingress-nginx";
const CHART_NAME: &str = "ingress-nginx";
const NAMESPACE: &str = "default";

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

    let template = helm::template(CHART_NAME, CHART_VERSION, NAMESPACE, out_path)?;
    write!(&mut file, "{}", template)?;
    Ok(())
}
