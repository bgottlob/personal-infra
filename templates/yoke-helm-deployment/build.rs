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

fn main() -> anyhow::Result<()> {
    helm::ensure_repo(REPO_NAME, REPO_URL)?;

    let out_dir = env::var("OUT_DIR")?;
    let out_path = Path::new(&out_dir);

    helm::pull(Some(REPO_NAME), CHART_NAME, CHART_VERSION, out_path)?;

    let mut out_file = BufWriter::new(
        File::create(out_path.join("helm-output.yaml"))?
    );

    let template = helm::template(CHART_NAME, CHART_VERSION, NAMESPACE, helm::TemplateOptions {
        release_name: "{{ project-name }}",
        values: Some(json!({
            // Fill these in with the values to be passed to the Helm chart
            "example-number": 1234,
            "example-obj": {
                "example-str": "abcd",
            },
        })),
        ..Default::default()
    }, out_path)?;
    write!(&mut out_file, "{}", template)?;

    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
