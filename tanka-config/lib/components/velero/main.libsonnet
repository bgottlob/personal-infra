local k = import 'github.com/grafana/jsonnet-libs/ksonnet-util/kausal.libsonnet';
local tanka = import 'github.com/grafana/jsonnet-libs/tanka-util/main.libsonnet';
local helm = tanka.helm.new(std.thisFile);

local util = import 'util/main.libsonnet';

{
  all(namespace='velero', accessKeyId, secretAccessKey): {
    local secretName = 'velero-s3',
    local cloudCreds = std.format(
      |||
        [default]
        aws_access_key_id=%s
        aws_secret_access_key=%s
      |||,
      [accessKeyId, secretAccessKey]
    ),

    namespace: k.core.v1.namespace.new(namespace),

    chart: helm.template('velero', './charts/velero', {
      namespace: namespace,
      includeCrds: true,
      values: {
        // Deploy the node agent DaemonSet to perform file system backups
        deployNodeAgent: true,
        // Linode volumes do not support snapshotting
        snapshotsEnabled: false,
        credentials: {
          name: secretName,
          secretContents: { cloud: cloudCreds },
        },
        configuration: {
          backupStorageLocation: [{
            name: 'default',
            provider: 'aws',
            bucket: 'bgottlob-velero-backups',
            config: {
              region: 'us-east-1',
              s3Url: 'https://us-east-1.linodeobjects.com',
	      // Non-AWS S3 object storage does not support checksums; setting
	      // algorithm to empty string skips checksum verification
	      checksumAlgorithm: '',
            },
          }],
        },
        initContainers: [{
          name: 'velero-plugin-for-aws',
          image: 'velero/velero-plugin-for-aws:v1.9.2',
          volumeMounts: [{
            mountPath: '/target',
            name: 'plugins',
          }],
        }],
        resources: {
          requests: {
            cpu: '100m',
          },
        },
        nodeAgent: {
          resources: {
            requests: {
              cpu: '100m',
            },
          },
        },
      },
    }),
  },
}
