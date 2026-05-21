use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

const CHART_VERSION: &str = "v1.7.3";
const NAMESPACE: &str = "envoy-gateway-system";
const MAIN_CHART_URL: &str = "oci://docker.io/envoyproxy/gateway-helm";
const CRDS_CHART_URL: &str = "oci://docker.io/envoyproxy/gateway-crds-helm";

const GATEWAY_CONTROLLER_NAME: &str = "gateway.envoyproxy.io/public-controller";

fn main() -> anyhow::Result<()> {
    let out_dir = env::var("OUT_DIR")?;
    let out_path = Path::new(&out_dir);

    println!(
        "cargo:rustc-env=GATEWAY_CONTROLLER_NAME={}",
        GATEWAY_CONTROLLER_NAME,
    );

    let mut main_out_file = BufWriter::new(
        File::create(out_path.join("main-helm-output.yaml"))?
    );
    let mut crds_out_file = BufWriter::new(
        File::create(out_path.join("crds-helm-output.yaml"))?
    );

    helm::pull(None, "oci://docker.io/envoyproxy/gateway-helm", CHART_VERSION, out_path)?;
    helm::pull(None, "oci://docker.io/envoyproxy/gateway-crds-helm", CHART_VERSION, out_path)?;

    let main_template = helm::template(MAIN_CHART_URL, CHART_VERSION, NAMESPACE, helm::TemplateOptions {
        release_name: "envoy-gateway",
        set_values: HashMap::new(),
        values: Some(json!({
            "deployment": {
                "replicas": 2,
                "envoyGateway": {
                    "resources": {
                        "requests": {
                            "cpu": "100m",
                            "memory": "128Mi",
                        },
                        "limits": {
                            "cpu": "500m",
                            "memory": "512Mi"
                        },
                    },
                }
            },
            "podDisruptionBudget": {
                "enabled": true,
                "minAvailable": 1,
            },
            "config": {
                "envoyGateway": {
                    "gateway": {
                        "controllerName": GATEWAY_CONTROLLER_NAME,
                    },
                },
            },
        })),
        ..Default::default()
    }, out_path)?;
    write!(&mut main_out_file, "{}", main_template)?;

    let crds_template = helm::template(CRDS_CHART_URL, CHART_VERSION, NAMESPACE, helm::TemplateOptions {
        release_name: "envoy-gateway-crds",
        set_values: HashMap::from([("crds.envoyGateway.enabled", "true")]),
        ..Default::default()
    }, out_path)?;
    write!(&mut crds_out_file, "{}", crds_template)?;

    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
