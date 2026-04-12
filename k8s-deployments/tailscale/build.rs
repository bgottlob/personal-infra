use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use serde_json::json;

const CHART_NAME: &str = "tailscale-operator";
const CHART_VERSION: &str = "1.94.2";
const NAMESPACE: &str = "tailscale";
const REPO_NAME: &str = "tailscale";
const REPO_URL: &str = "https://pkgs.tailscale.com/helmcharts";

fn main() -> anyhow::Result<()> {
    helm::ensure_repo(REPO_NAME, REPO_URL)?;

    let out_dir = env::var("OUT_DIR")?;
    let out_path = Path::new(&out_dir);

    let secrets_path = std::env::var("SECRETS_FILE").unwrap_or("../../secrets.yaml".into());
    let secrets = secrets::decrypt_parse_secrets(&secrets_path)
        .unwrap_or_else(|_| panic!("Should parse {} file into Secrets struct", secrets_path));

    helm::pull(Some(REPO_NAME), CHART_NAME, CHART_VERSION, out_path)?;

    let mut out_file = BufWriter::new(
        File::create(out_path.join("helm-output.yaml"))?
    );

    let template = helm::template(CHART_NAME, CHART_VERSION, NAMESPACE, helm::TemplateOptions {
        release_name: "tailscale",
        set_values: HashMap::new(),
        values: Some(json!({
            "oauth": {
                "clientId": secrets.tailscale.oauth.client_id,
                "clientSecret": secrets.tailscale.oauth.client_secret,
            },
        })),
        ..Default::default()
    }, out_path)?;
    write!(&mut out_file, "{}", template)?;

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", secrets_path);

    Ok(())
}
