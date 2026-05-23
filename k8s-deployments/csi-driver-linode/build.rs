use std::env;
use std::collections::HashMap;
use serde_json::json;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

const CHART_VERSION: &str = "v1.1.3";
const REPO_NAME: &str = "linode-csi";
const REPO_URL: &str = "https://linode.github.io/linode-blockstorage-csi-driver/";
const CHART_NAME: &str = "linode-blockstorage-csi-driver";
const NAMESPACE: &str = "kube-system";
const RELEASE_NAME: &str = "csi-driver-linode";

fn main() -> anyhow::Result<()> {
    helm::ensure_repo(REPO_NAME, REPO_URL)?;
    let out_dir = env::var("OUT_DIR")?;
    let out_path = Path::new(&out_dir);

    helm::pull(Some(REPO_NAME), CHART_NAME, CHART_VERSION, out_path)?;

    let mut file = BufWriter::new(
        File::create(out_path.join("helm-output.yaml"))?
    );

    let template = helm::template(CHART_NAME, CHART_VERSION, NAMESPACE, helm::TemplateOptions {
        release_name: RELEASE_NAME,
        set_values: HashMap::from([
            // Use a dedicated secret name to avoid conflict with the `linode`
            // secret that LKE injects in kube-system with its own credentials.
            ("secretRef.name", "linode-csi"),
            ("secretRef.apiTokenRef", "token"),
            ("secretRef.regionRef", "region"),
        ]),
        values: Some(json!({
            // TODO: blocked by upstream chart bug — daemonset.yaml uses nindent 8 instead of
            // nindent 10, so limits/requests render as invalid top-level container fields rather
            // than under resources:. Uncomment once the chart is fixed.
            // "csiLinodePlugin": {
            //     "resources": {
            //         "requests": { "cpu": "15m", "memory": "64Mi" },
            //         "limits":   { "cpu": "50m", "memory": "64Mi" },
            //     },
            // },
            "csiProvisioner": {
                "resources": {
                    "requests": { "cpu": "15m", "memory": "32Mi" },
                    "limits":   { "cpu": "50m", "memory": "32Mi" },
                },
            },
            "csiAttacher": {
                "resources": {
                    "requests": { "cpu": "10m", "memory": "32Mi" },
                    "limits":   { "cpu": "50m", "memory": "32Mi" },
                },
            },
            "csiResizer": {
                "resources": {
                    "requests": { "cpu": "10m", "memory": "32Mi" },
                    "limits":   { "cpu": "50m", "memory": "32Mi" },
                },
            },
            "csiNodeDriverRegistrar": {
                "resources": {
                    "requests": { "cpu": "10m", "memory": "32Mi" },
                    "limits":   { "cpu": "50m", "memory": "32Mi" },
                },
            },
        })),
        ..Default::default()
    }, out_path)?;
    write!(&mut file, "{}", template)?;
    Ok(())
}
