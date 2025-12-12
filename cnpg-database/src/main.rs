mod cnpg;

use std::collections::BTreeMap;

use crate::cnpg::*;
use k8s_openapi::api::core::v1::Secret;
use kube::core::ObjectMeta;
use kube_builder::secret::SecretBuilder;

const CNPG_CLUSTER_NAME: &str = "main-db";
const SUPER_USER_CREDS_SECRET_NAME: &str = "super-user-creds";
const APP_USER_CREDS_SECRET_NAME: &str = "app-user-creds";
const IMAGE_CATALOG_NAME: &str = "postgresql";
const S3_CREDS_SECRET_NAME: &str = "backup-s3-creds";
const OBJECT_STORE_NAME: &str = "linode-store";
const BARMAN_PLUGIN_NAME: &str = "barman-cloud.cloudnative-pg.io";

fn main() -> anyhow::Result<()> {
    let mut resources = Vec::new();
    resources.push(serde_json::to_value(create_app_user_secret())?);
    resources.push(serde_json::to_value(create_super_user_secret())?);
    resources.push(serde_json::to_value(create_s3_secret())?);
    resources.push(serde_json::to_value(create_image_catalog())?);
    resources.push(serde_json::to_value(create_database_cluster())?);
    resources.push(serde_json::to_value(create_scheduled_backup())?);
    resources.push(serde_json::to_value(create_object_store())?);

    resources.push(serde_json::to_value(create_database(
        String::from("wallabag"), String::from(env!("APP_USER_USERNAME")),
    ))?);

    resources.push(serde_json::to_value(create_database(
        String::from("vikunja"), String::from(env!("APP_USER_USERNAME")),
    ))?);

    resources.push(serde_json::to_value(create_database(
        String::from("miniflux"), String::from(env!("APP_USER_USERNAME")),
    ))?);

    println!("{}", serde_json::to_string(&resources).unwrap());
    Ok(())
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

fn create_app_user_secret() -> Secret {
    SecretBuilder::new()
        .name(APP_USER_CREDS_SECRET_NAME)
        .value("username", env!("APP_USER_USERNAME"))
        .value("password", env!("APP_USER_PASSWORD"))
        .build()
        .expect("app user secret should be built")
}

fn create_super_user_secret() -> Secret {
    SecretBuilder::new()
        .name(SUPER_USER_CREDS_SECRET_NAME)
        .value("username", env!("SUPER_USER_USERNAME"))
        .value("password", env!("SUPER_USER_PASSWORD"))
        .build()
        .expect("super user secret should be built")
}

fn create_s3_secret() -> Secret {
    SecretBuilder::new()
        .name(S3_CREDS_SECRET_NAME)
        .value("access_key_id", env!("ACCESS_KEY_ID"))
        .value("secret_key", env!("SECRET_KEY"))
        .build()
        .expect("s3 secret should be built")
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

fn create_database_cluster() -> Cluster {
    let name = String::from(CNPG_CLUSTER_NAME);
    let pg_major_version = 18;

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

            bootstrap: Some(ClusterBootstrap {

                initdb: Some(ClusterBootstrapInitdb {
                    database: Some(String::from("app")),
                    // TODO make this a var
                    owner: Some(String::from("app")),
                    secret: Some(ClusterBootstrapInitdbSecret {
                        name: APP_USER_CREDS_SECRET_NAME.to_string(),
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }),

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
                     parameters: Some(BTreeMap::from([
                         (String::from("barmanObjectName"), OBJECT_STORE_NAME.to_string()),
                     ])),
                 }
            ]),

            /*
            backup: Some(ClusterBackup {
                retention_policy: Some(String::from("14d")),
                barman_object_store: Some(ClusterBackupBarmanObjectStore {
                    destination_path: String::from("s3:://bgottlob-db-backup/backups"),
                    endpoint_url: Some(String::from("https://us-east-1.linodeobjects.com")),
                    // For some reason, maybe due to Linode's limitations, I needed to
                    // turn off encryption in data and wal to get this to work
                    // https://github.com/cloudnative-pg/cloudnative-pg/discussions/4376#discussioncomment-11566074
                    data: Some(ClusterBackupBarmanObjectStoreData {
                        compression: Some(ClusterBackupBarmanObjectStoreDataCompression::Gzip),
                        jobs: Some(2),
                        encryption: None,
                        ..Default::default()
                    }),
                    wal: Some(ClusterBackupBarmanObjectStoreWal {
                        compression: Some(ClusterBackupBarmanObjectStoreWalCompression::Gzip),
                        max_parallel: Some(1),
                        encryption: None,
                        ..Default::default()
                    }),
                    s3_credentials: Some(ClusterBackupBarmanObjectStoreS3Credentials {
                        access_key_id: Some(ClusterBackupBarmanObjectStoreS3CredentialsAccessKeyId {
                            name: String::from(S3_CREDS_SECRET_NAME),
                            key: String::from("access_key_id")
                        }),
                        secret_access_key: Some(ClusterBackupBarmanObjectStoreS3CredentialsSecretAccessKey {
                            name: String::from(S3_CREDS_SECRET_NAME),
                            key: String::from("secret_key")
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            */

            superuser_secret: Some(ClusterSuperuserSecret {
                name: String::from(SUPER_USER_CREDS_SECRET_NAME),
            }),

            ..Default::default()
        },
        ..Default::default()
    }
}

fn create_scheduled_backup() -> ScheduledBackup {
    ScheduledBackup {
        metadata: ObjectMeta {
            name: Some(String::from("daily-backup")),
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
                name: String::from(CNPG_CLUSTER_NAME),
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
