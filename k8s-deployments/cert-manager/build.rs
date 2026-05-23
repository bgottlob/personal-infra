use std::collections::HashMap;
use std::env;
use serde_json::json;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

const CHART_VERSION: &str = "v1.17.4";
const REPO_NAME: &str = "jetstack";
const REPO_URL: &str = "https://charts.jetstack.io";
const CHART_NAME: &str = "cert-manager";
const NAMESPACE: &str = "cert-manager";

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
        set_values: HashMap::from([
            ("crds.enabled", "true"),
            ("config.apiVersion", "controller.config.cert-manager.io/v1alpha1"),
            ("config.kind", "ControllerConfiguration"),
            ("config.enableGatewayAPI", "true"),
        ]),
        values: Some(json!({
            "resources": { // cert-manager-controller
                "requests": { "cpu": "25m", "memory": "64Mi" },
                "limits":   { "cpu": "100m", "memory": "64Mi" },
            },
            "cainjector": {
                "resources": {
                    "requests": { "cpu": "25m", "memory": "128Mi" },
                    "limits":   { "cpu": "100m", "memory": "128Mi" },
                },
            },
            "webhook": {
                "resources": {
                    "requests": { "cpu": "25m", "memory": "32Mi" },
                    "limits":   { "cpu": "100m", "memory": "32Mi" },
                },
            },
        })),
        ..Default::default()
    }, out_path)?;
    write!(&mut file, "{}", template)?;
    Ok(())
}
