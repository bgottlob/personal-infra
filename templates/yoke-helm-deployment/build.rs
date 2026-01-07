use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use serde_json::json;

const CHART_NAME: &str = "{{ chart_name }}";
const CHART_VERSION: &str = "{{ chart_version }}";
const NAMESPACE: &str = "{{ namespace }}";
const REPO_NAME: &str = "{{ chart_repo_name }}";
const REPO_URL: &str = "{{ chart_repo_url }}";

fn values() -> serde_json::Value {
    json!({
        // Fill these in with the values to be passed to the Helm chart
        "example-number": 1234,
        "example-obj": {
            "example-str": "abcd",
        },
    })
}

fn main() -> anyhow::Result<()> {
    if !helm::repo_exists(REPO_NAME, REPO_URL) {
        helm::add_repo(REPO_NAME, REPO_URL)
    }
    let out_dir = env::var("OUT_DIR")?;
    let out_path = Path::new(&out_dir);

    let secrets_path = std::env::var("SECRETS_FILE").unwrap_or("../secrets.yaml".into());
    let secrets = secrets::decrypt_parse_secrets(&secrets_path)
        .unwrap_or_else(|_| panic!("Should parse {} file into Secrets struct", secrets_path));

    let example_secret = "example";
    println!(
        "cargo:rustc-env=EXAMPLE_SECRET={}",
        example_secret,
    );

    helm::pull(Some(REPO_NAME), CHART_NAME, CHART_VERSION, out_path)?;

    let mut out_file = BufWriter::new(
        File::create(out_path.join("helm-output.yaml"))?
    );

    let values_path = out_path.join("values.yaml");
    {
        let mut values_file = BufWriter::new(
            File::create(values_path.clone())?
        );

        let values = values();
        let values_str = serde_norway::to_string(&values).expect("values should serialize to yaml");
        write!(&mut values_file, "{}", values_str)?;
    }
    // using the scope above closes the values file so that it is fully written
    // to before helm template needs to use it

    let template = helm::template(
        CHART_NAME,
        CHART_VERSION,
        NAMESPACE,
        Some("{{ project-name }}"),
        None,
        Some(Path::new("values.yaml")),
        out_path
    )?;
    write!(&mut out_file, "{}", template)?;

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", secrets_path);

    Ok(())
}
