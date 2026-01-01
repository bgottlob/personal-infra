use std::env;
use kube::core::ObjectMeta;
use serde::Deserialize;

mod velero;
use velero::schedule::*;

fn create_schedule() -> Schedule {
    Schedule {
        metadata: ObjectMeta {
            name: Some(String::from("all-resources-volumes")),
            ..Default::default()
        },
        spec: ScheduleSpec {
            schedule: String::from("0 */12 * * *"),
            template: ScheduleTemplate {
                included_namespaces: Some(vec![
                    String::from("*")
                ]),
                excluded_namespaces: Some(vec![
                    String::from("main-db"),
                    String::from("cnpg-system"),
                ]),
                // Data expires after a week
                ttl: Some(String::from("168h0m0s")),
                // Back up file systems of volumes
                default_volumes_to_fs_backup: Some(true),
                // Linode does not support volume snapshots
                snapshot_volumes: Some(false),
                // Backup storage location created by helm chart values
                storage_location: Some(String::from("default")),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    }
}

fn main() -> anyhow::Result<()> {
    let helm_out_str = include_str!(concat!(env!("OUT_DIR"), "/helm-output.yaml"));
    let mut resources: Vec<serde_json::Value> = Vec::new();
    for document in serde_norway::Deserializer::from_str(helm_out_str) {
        let value = serde_norway::Value::deserialize(document)?;
        if value != serde_norway::Value::Null {
            resources.push(serde_json::to_value(value)?);
        }
    }
    let schedule = serde_json::to_value(create_schedule())?;
    resources.push(schedule);
    println!("{}", serde_json::to_string(&resources).unwrap());
    Ok(())
}
