use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

const CHART_VERSION: &str = "v1.6.1";
const NAMESPACE: &str = "envoy-gateway-system";
const MAIN_CHART_URL: &str = "oci://docker.io/envoyproxy/gateway-helm";
const CRDS_CHART_URL: &str = "oci://docker.io/envoyproxy/gateway-crds-helm";

const GATEWAY_CONTROLLER_NAME: &str = "gateway.envoyproxy.io/public-controller";

fn create_main_values_file(out_path: &Path) -> anyhow::Result<String> {
    let values = json!({
        "config": {
            "envoyGateway": {
                "gateway": {
                    "controllerName": GATEWAY_CONTROLLER_NAME,
                },
            },
        },
    });

    let values_path = out_path.join("main-values.yaml");
    let mut values_file = BufWriter::new(
        File::create(values_path.clone())?
    );

    let values_str = serde_norway::to_string(&values).expect("values should serialize to yaml");
    write!(&mut values_file, "{}", values_str)?;

    Ok(String::from("main-values.yaml"))
}

fn main() -> anyhow::Result<()> {
    let out_dir = env::var("OUT_DIR")?;
    let out_path = Path::new(&out_dir);

    let main_values_file_name = create_main_values_file(out_path)?;
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

    let main_template = helm::template(
        MAIN_CHART_URL,
        CHART_VERSION,
        NAMESPACE,
        Some("envoy-gateway"),
        None,
        Some(Path::new(&main_values_file_name)),
        out_path
    )?;
    write!(&mut main_out_file, "{}", main_template)?;

    let crds_template = helm::template(
        CRDS_CHART_URL,
        CHART_VERSION,
        NAMESPACE,
        Some("envoy-gateway-crds"),
        Some(HashMap::from([
           ("crds.envoyGateway.enabled", "true"),
        ])),
        None,
        out_path
    )?;
    write!(&mut crds_out_file, "{}", crds_template)?;

    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
