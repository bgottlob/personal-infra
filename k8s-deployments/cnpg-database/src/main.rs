mod cnpg;

use std::collections::BTreeMap;

use crate::cnpg::*;
use kube::core::ObjectMeta;
use kube_builder::prelude::*;

const CNPG_CLUSTER_NAME: &str = "main-db";
const APP_USER_CREDS_SECRET_NAME: &str = "app-user-creds";
const SUPER_USER_CREDS_SECRET_NAME: &str = "super-user-creds";
const IMAGE_CATALOG_NAME: &str = "postgresql";
const S3_CREDS_SECRET_NAME: &str = "backup-s3-creds";
const OBJECT_STORE_NAME: &str = "linode-store";
const BARMAN_PLUGIN_NAME: &str = "barman-cloud.cloudnative-pg.io";
// Must match Cluster.spec.bootstrap.initdb.owner
const APP_USER_USERNAME: &str = "app";

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value_t = false)]
    restore: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let is_restore = args.restore;
    let sealed_secrets = read_sealed_secrets_from_stdin()?;
    let mut resources: Vec<Vec<serde_json::Value>> = Vec::new();

    if !is_restore && !sealed_secrets.is_empty() {
        resources.push(sealed_secrets);
    }

    let mut other: Vec<serde_json::Value> = Vec::new();

    other.push(serde_json::to_value(create_database_cluster(is_restore))?);
    other.push(serde_json::to_value(create_scheduled_backup(is_restore))?);

    if !is_restore {
        other.push(serde_json::to_value(create_image_catalog())?);
        other.push(serde_json::to_value(create_object_store())?);

        other.push(serde_json::to_value(create_database(
            String::from("wallabag"), String::from(APP_USER_USERNAME),
        ))?);

        other.push(serde_json::to_value(create_database(
            String::from("vikunja"), String::from(APP_USER_USERNAME),
        ))?);

        other.push(serde_json::to_value(create_database(
            String::from("miniflux"), String::from(APP_USER_USERNAME),
        ))?);
    }

    resources.push(other);
    println!("{}", serde_json::to_string(&resources)?);
    Ok(())
}

fn barman_plugin_params(is_restore: bool) -> BTreeMap<String, String> {
    let mut server_name = CNPG_CLUSTER_NAME.to_string();
    if is_restore {
        server_name.push_str("-restore");
    }

    BTreeMap::from([
        (String::from("barmanObjectName"), OBJECT_STORE_NAME.to_string()),
        (String::from("serverName"), server_name),
    ])
}

fn create_database(name: String, owner: String) -> Database {
    Database {
        metadata: ObjectMeta {
            name: Some(name.clone()),
            ..Default::default()
        },
        spec: DatabaseSpec {
            name: name,
            owner: owner,
            cluster: DatabaseCluster {
                name: Some(CNPG_CLUSTER_NAME.to_string())
            },
            ..Default::default()
        },
        ..Default::default()
    }
}

fn create_object_store() -> ObjectStore {
    ObjectStore {
        metadata: ObjectMeta {
            name: Some(OBJECT_STORE_NAME.to_string()),
            ..Default::default()
        },
        spec: ObjectStoreSpec {
            // Fixes 411 length required error
            // https://cloudnative-pg.io/plugin-barman-cloud/docs/object_stores/#s3-compatible-storage-providers
            instance_sidecar_configuration: Some(ObjectStoreInstanceSidecarConfiguration {
                env: Some(vec![
                     ObjectStoreInstanceSidecarConfigurationEnv {
                         name: String::from("AWS_REQUEST_CHECKSUM_CALCULATION"),
                         value: Some(String::from("when_required")),
                         ..Default::default()
                     },
                     ObjectStoreInstanceSidecarConfigurationEnv {
                         name: String::from("AWS_RESPONSE_CHECKSUM_VALIDATION"),
                         value: Some(String::from("when_required")),
                         ..Default::default()
                     },
                ]),
                ..Default::default()
            }),
            configuration: ObjectStoreConfiguration {
                    destination_path: String::from("s3://bgottlob-db-backup"),
                    endpoint_url: Some(String::from("https://us-east-1.linodeobjects.com")),
                    // For some reason, maybe due to Linode's limitations, I needed to
                    // turn off encryption in data and wal to get this to work
                    // https://github.com/cloudnative-pg/cloudnative-pg/discussions/4376#discussioncomment-11566074
                    data: Some(ObjectStoreConfigurationData {
                        compression: Some(ObjectStoreConfigurationDataCompression::Gzip),
                        jobs: Some(2),
                        encryption: None,
                        ..Default::default()
                    }),
                    wal: Some(ObjectStoreConfigurationWal {
                        compression: Some(ObjectStoreConfigurationWalCompression::Gzip),
                        max_parallel: Some(1),
                        encryption: None,
                        ..Default::default()
                    }),
                    s3_credentials: Some(ObjectStoreConfigurationS3Credentials {
                        access_key_id: Some(ObjectStoreConfigurationS3CredentialsAccessKeyId {
                            name: String::from(S3_CREDS_SECRET_NAME),
                            key: String::from("access_key_id")
                        }),
                        secret_access_key: Some(ObjectStoreConfigurationS3CredentialsSecretAccessKey {
                            name: String::from(S3_CREDS_SECRET_NAME),
                            key: String::from("secret_key")
                        }),
                        region: Some(ObjectStoreConfigurationS3CredentialsRegion {
                            name: String::from(S3_CREDS_SECRET_NAME),
                            key: String::from("region")
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
            },
            retention_policy: Some(String::from("14d"))
        },
        ..Default::default()
    }
}

fn create_database_cluster(is_restore: bool) -> Cluster {
    let mut name = CNPG_CLUSTER_NAME.to_string();
    if is_restore {
        name.push_str("-restore");
    }
    let pg_major_version = 18;

    let bootstrap = if is_restore {
        ClusterBootstrap {
            recovery: Some(ClusterBootstrapRecovery {
                source: Some(String::from("origin")),
                ..Default::default()
            }),
            ..Default::default()
        }
    } else {
        ClusterBootstrap {
            initdb: Some(ClusterBootstrapInitdb {
                database: Some(String::from("app")),
                owner: Some(String::from("app")),
                secret: Some(ClusterBootstrapInitdbSecret {
                    name: APP_USER_CREDS_SECRET_NAME.to_string(),
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    };

    let external_clusters = if is_restore {
        Some(vec![ClusterExternalClusters {
            name: String::from("origin"),
            plugin: Some(ClusterExternalClustersPlugin {
                name: BARMAN_PLUGIN_NAME.to_string(),
                parameters: Some(barman_plugin_params(false)), // this should always be false to
                                                               // restore from main-db
                ..Default::default()
            }),
            ..Default::default()
        }])
    } else {
        None
    };

    Cluster {
        metadata: ObjectMeta {
            name: Some(name),
            ..Default::default()
        },
        spec: ClusterSpec {
            description: Some(String::from("Postgres cluster for all applications running on this k8s cluster")),
            instances: 1,
            image_catalog_ref: Some(ClusterImageCatalogRef {
                api_group: Some(String::from("postgresql.cnpg.io")),
                kind: String::from("ImageCatalog"),
                name: String::from(IMAGE_CATALOG_NAME),
                major: pg_major_version,
            }),

            bootstrap: Some(bootstrap),

            external_clusters,

            storage: Some(ClusterStorage {
                size: Some(String::from("20Gi")),
                storage_class: Some(String::from("linode-block-storage-retain-encrypted")),
                resize_in_use_volumes: Some(true),
                ..Default::default()

            }),

            plugins: Some(vec![
                 ClusterPlugins {
                     enabled: Some(true),
                     name: BARMAN_PLUGIN_NAME.to_string(),
                     is_wal_archiver: Some(true),
                     parameters: Some(barman_plugin_params(is_restore)),
                 }
            ]),

            superuser_secret: Some(ClusterSuperuserSecret {
                name: String::from(SUPER_USER_CREDS_SECRET_NAME),
            }),

            ..Default::default()
        },
        ..Default::default()
    }
}

fn create_scheduled_backup(is_restore: bool) -> ScheduledBackup {
    let mut name = String::from("daily-backup");
    if is_restore {
        name.push_str("-restore");
    }

    let mut cluster_name = CNPG_CLUSTER_NAME.to_string();
    if is_restore {
        cluster_name.push_str("-restore");
    }

    ScheduledBackup {
        metadata: ObjectMeta {
            name: Some(name),
            ..Default::default()
        },
        spec: ScheduledBackupSpec {
            backup_owner_reference: Some(ScheduledBackupBackupOwnerReference::_Self),
            // Every night at 8 pm
            schedule: String::from("0 0 20 * * *"),
            method: Some(ScheduledBackupMethod::Plugin),
            plugin_configuration: Some(ScheduledBackupPluginConfiguration {
                name: BARMAN_PLUGIN_NAME.to_string(),
                ..Default::default()
            }),
            cluster: ScheduledBackupCluster {
                name: cluster_name,
            },
            ..Default::default()
        },
        ..Default::default()
    }
}

fn create_image_catalog() -> ImageCatalog {
    // https://github.com/cloudnative-pg/artifacts/tree/main/image-catalogs
    ImageCatalog {
        metadata: ObjectMeta {
            name: Some(String::from(IMAGE_CATALOG_NAME)),
            ..Default::default()
        },
        spec: ImageCatalogSpec {
            images: vec![
                ImageCatalogImages {
                    major: 18,
                    image: String::from("ghcr.io/cloudnative-pg/postgresql:18.1-system-trixie")
                }
            ]
        }
    }
}
