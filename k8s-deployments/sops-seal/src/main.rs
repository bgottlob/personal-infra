mod input;
mod path_transform;
mod seal;
mod sops;

use std::path::PathBuf;
use clap::{Parser, ValueEnum};

use crate::path_transform::build_secret;

#[derive(Parser)]
#[command(about = "Seal sops-encrypted secrets into Kubernetes SealedSecrets")]
struct Args {
    /// Path to Secret template file, or - for stdin
    template: PathBuf,

    /// Path to sops-encrypted secrets file
    #[arg(short, long, env = "SECRETS_FILE")]
    secrets: PathBuf,

    /// Kubeseal public key certificate for offline sealing
    #[arg(long, env = "KUBESEAL_CERT")]
    cert: Option<PathBuf>,

    /// Output format for the SealedSecrets
    #[arg(short, long, default_value = "yaml")]
    format: Format,

    /// Output the plain Secret before sealing, for debugging
    #[arg(long)]
    unsealed: bool,
}

#[derive(Clone, ValueEnum)]
enum Format {
    Json,
    Yaml,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let secrets = sops::decrypt(args.secrets)?;
    let templates = input::read(args.template)?;

    match args.format {
        Format::Json => {
            let mut sealed_secrets: Vec<serde_json::Value> = Vec::new();
            for template in templates {
                let secret = build_secret(template, &secrets)?;
                let secret_str = serde_norway::to_string(&secret)?;
                let sealed_str = seal::seal_token(secret_str, "json")?;
                sealed_secrets.push(serde_json::from_str(&sealed_str)?);
            }
            println!("{}", serde_json::to_string(&sealed_secrets)?);
        }
        Format::Yaml => {
            for template in templates {
                let secret = build_secret(template, &secrets)?;
                let secret_str = serde_norway::to_string(&secret)?;
                let sealed = seal::seal_token(secret_str, "yaml")?;
                print!("{sealed}");
            }
        }
    }

    Ok(())
}
