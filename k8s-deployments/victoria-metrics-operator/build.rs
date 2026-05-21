use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

const CHART_VERSION: &str = "0.57.1";
const REPO_NAME: &str = "vm";
const REPO_URL: &str = "https://victoriametrics.github.io/helm-charts/";
const CHART_NAME: &str = "victoria-metrics-operator";
const NAMESPACE: &str = "victoria-metrics-operator";

fn main() -> anyhow::Result<()> {
    helm::ensure_repo(REPO_NAME, REPO_URL)?;
    let out_dir = env::var("OUT_DIR")?;
    let out_path = Path::new(&out_dir);

    helm::pull(Some(REPO_NAME), CHART_NAME, CHART_VERSION, out_path)?;

    {
        let mut file = BufWriter::new(
            File::create(out_path.join("helm-output.yaml"))?
        );

        let template = helm::template(CHART_NAME, CHART_VERSION, NAMESPACE, helm::TemplateOptions {
            release_name: "vmo",
            set_values: HashMap::from([
                ("nameOverride", "vmo"),
                // https://github.com/VictoriaMetrics/helm-charts/issues/2420#issuecomment-3341540003
                ("crds.plain", "true"),
                ("crds.upgrade.enabled", "true"),
                ("crds.upgrade.forceConflicts", "true"),
                // Use cert-manager to create and rotate the vmo-validation webhook TLS secret
                ("admissionWebhooks.certManager.enabled", "true"),
            ]),
            ..Default::default()
        }, out_path)?;
        write!(&mut file, "{}", template)?;
    }

    Ok(())
}


