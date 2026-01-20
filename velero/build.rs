use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use serde_json::json;

const CHART_VERSION: &str = "11.3.1";
const REPO_NAME: &str = "vmwareTanzu";
const REPO_URL: &str = "https://vmware-tanzu.github.io/helm-charts";
const CHART_NAME: &str = "velero";
const NAMESPACE: &str = "velero";
const VELERO_PLUGIN_VERSION: &str = "1.13.1";

fn values(access_key_id: String, secret_access_key: String) -> serde_json::Value {
    let secret_name = "velero-s3";
    let cloud_creds = format!(
        concat!(
            "[default]\n",
            "aws_access_key_id={}\n",
            "aws_secret_access_key={}\n"
        ),
        access_key_id,
        secret_access_key,
    );

    let velero_plugin_image = format!("velero/velero-plugin-for-aws:v{}", VELERO_PLUGIN_VERSION);

    json!({
        // Deploy the node agent DaemonSet to perform file system backups
        "deployNodeAgent": true,
        // Linode volumes do not support snapshotting
        "snapshotsEnabled": false,
        // File system backups
        "backupsEnabled": true,
        "credentials": {
            "name": secret_name,
            "secretContents": {
                "cloud": cloud_creds,
            },
        },
        "configuration": {
            "backupStorageLocation": [{
                "name": "default",
                "provider": "aws",
                "bucket": "bgottlob-velero-backups",
                "config": {
                    "region": "us-east-1",
                    "s3Url": "https://us-east-1.linodeobjects.com",
                    // Non-AWS S3 object storage does not support checksums; setting
                    // algorithm to empty string skips checksum verification
                    "checksumAlgorithm": "",
                },
            }],
        },
        "initContainers": [{
            "name": "velero-plugin-for-aws",
            "image": velero_plugin_image,
            "volumeMounts": [{
                "mountPath": "/target",
                "name": "plugins",
            }],
        }],
        "resources": {
            "requests": {
                "cpu": "100m",
                "memory": "128Mi",
            },
        },
        "nodeAgent": {
            "resources": {
                "requests": {
                    "cpu": "5m",
                    "memory": "100Mi"
                },
                "limits": {
                    "cpu": "500m",
                    "memory": "512Mi"
                },
            },
        },
        "kubectl": {
            "image": {
                "tag": "1.33.4"
            }
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

    helm::pull(Some(REPO_NAME), CHART_NAME, CHART_VERSION, out_path)?;

    let mut out_file = BufWriter::new(
        File::create(out_path.join("helm-output.yaml"))?
    );

    let values_path = out_path.join("values.yaml");
    {
        let mut values_file = BufWriter::new(
            File::create(values_path.clone())?
        );

        let values = values(
            secrets.velero.bucket.access_key_id,
            secrets.velero.bucket.secret_key
        );
        let values_str = serde_norway::to_string(&values).expect("values should serialize to yaml");
        write!(&mut values_file, "{}", values_str)?;
    }
    // using the scope above closes the values file so that it is fully written
    // to before helm template needs to use it

    let template = helm::template(CHART_NAME, CHART_VERSION, NAMESPACE, Some("velero"), None, Some(Path::new("values.yaml")), out_path)?;
    write!(&mut out_file, "{}", template)?;

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", secrets_path);

    Ok(())
}
