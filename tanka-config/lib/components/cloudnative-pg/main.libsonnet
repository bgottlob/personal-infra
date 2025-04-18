local tanka = import 'github.com/grafana/jsonnet-libs/tanka-util/main.libsonnet';
local helm = tanka.helm.new(std.thisFile);

{
  all(
    backupBucketAccessKey,
    backupBucketSecretKey,
    operatorNamespace='cnpg-system',
    databaseNamespace='cnpg-database',
    databases=[],
  ): {
    local databaseName = databaseNamespace + '-cluster',

    operator: helm.template('cnpg-operator', './charts/cloudnative-pg', {
      namespace: operatorNamespace,
      includeCrds: true,
    }),

    // Some docs for cluster configuration
    // https://github.com/cloudnative-pg/charts/blob/main/charts/cluster/docs/Getting%20Started.md#creating-a-cluster-configuration
    // https://github.com/cloudnative-pg/charts/tree/main/charts/cluster/examples
    database: helm.template(databaseName, './charts/cluster', {
      namespace: databaseNamespace,
      values: {
        type: 'postgresql',
        mode: 'standalone',
        version: {
          postgresql: '17'
        },
        cluster: {
          instances: 3,
          storage: {
            size: '10Gi'
          }
        },
        backups: {
          enabled: true,
          endpointURL: 'https://us-east-1.linodeobjects.com',
          provider: 's3',
          s3: {
            region: 'us-east-1',
            bucket: 'bgottlob-db-backup',
            path: '/backups',
            accessKey: backupBucketAccessKey,
            secretKey: backupBucketSecretKey,
          },
          scheduledBackups: [{
            name: 'daily-backup',
            // Every night at 8 pm
            schedule: '0 0 20 * * *',
            backupOwnerReference: 'self',
            method: 'barmanObjectStore',
          }],
          retentionPolicy: '30d',
          // For some reason, maybe due to Linode's limitations, I needed to
          // turn off encryption to get this to work
          // https://github.com/cloudnative-pg/cloudnative-pg/discussions/4376#discussioncomment-11566074
          wal: { encryption: '' },
          data: { encryption: '' },
        },
      }
    }),

    databases: [
      {
        apiVersion: 'postgresql.cnpg.io/v1',
        kind: 'Database',
        metadata: {
          name: dbName,
          namespace: databaseNamespace,
        },
        spec: {
          name: dbName,
          owner: 'app',
          cluster: {
            name: databaseName
          }
        }
      }
      for dbName in databases
    ]
  }
}
