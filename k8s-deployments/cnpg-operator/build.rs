use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

const REPO_NAME: &str = "cnpg";
const REPO_URL: &str = "https://cloudnative-pg.github.io/charts";
const NAMESPACE: &str = "cnpg-system";

const BARMAN_CHART_NAME: &str = "plugin-barman-cloud";
const BARMAN_CHART_VERSION: &str = "0.3.1";
const CNPG_CHART_NAME: &str = "cloudnative-pg";
const CNPG_CHART_VERSION: &str = "0.27.0";

fn main() -> anyhow::Result<()> {
    helm::ensure_repo(REPO_NAME, REPO_URL)?;
    let out_dir = env::var("OUT_DIR")?;
    let out_path = Path::new(&out_dir);

    helm::pull(Some(REPO_NAME), CNPG_CHART_NAME, CNPG_CHART_VERSION, out_path)?;
    helm::pull(Some(REPO_NAME), BARMAN_CHART_NAME, BARMAN_CHART_VERSION, out_path)?;

    let mut cnpg_file = BufWriter::new(
        File::create(out_path.join("cnpg-helm-output.yaml"))?
    );

    let mut barman_file = BufWriter::new(
        File::create(out_path.join("barman-helm-output.yaml"))?
    );

    let cnpg_template = helm::template(CNPG_CHART_NAME, CNPG_CHART_VERSION, NAMESPACE, helm::TemplateOptions {
        release_name: CNPG_CHART_NAME,
        set_values: HashMap::new(),
        values: None,
    }, out_path)?;
    let barman_template = helm::template(BARMAN_CHART_NAME, BARMAN_CHART_VERSION, NAMESPACE, helm::TemplateOptions {
        release_name: BARMAN_CHART_NAME,
        set_values: HashMap::new(),
        values: None,
    }, out_path)?;

    write!(&mut cnpg_file, "{}", cnpg_template)?;
    write!(&mut barman_file, "{}", barman_template)?;

    Ok(())
}
