use std::env;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

const CHART_VERSION: &str = "v1.0.11";
const REPO_NAME: &str = "linode-csi";
const REPO_URL: &str = "https://linode.github.io/linode-blockstorage-csi-driver/";
const CHART_NAME: &str = "linode-blockstorage-csi-driver";
const NAMESPACE: &str = "kube-system";
const RELEASE_NAME: &str = "csi-driver-linode";

fn main() -> anyhow::Result<()> {
    let secrets_path = std::env::var("SECRETS_FILE").unwrap_or("../../secrets.yaml".into());
    let secrets = secrets::decrypt_parse_secrets(&secrets_path)
        .unwrap_or_else(|_| panic!("Should parse {} file into Secrets struct", secrets_path));

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
            ("region", "us-east"),
            ("apiToken", secrets.linode.csi_driver_token.as_str()),
        ]),
        values: None,
    }, out_path)?;
    write!(&mut file, "{}", template)?;
    Ok(())
}
