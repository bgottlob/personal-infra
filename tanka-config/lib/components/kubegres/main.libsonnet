local k = import 'github.com/grafana/jsonnet-libs/ksonnet-util/kausal.libsonnet';
local util = import 'util/main.libsonnet';

local namespace = k.core.v1.namespace;

{
  _config:: {
    kubegresName: 'mypostgres',
    passwordSecretName: 'mypostgres-secret',
    postgresImage: {
      name: 'postgres',
      tag: '14.6',
    },
    databaseSize: '200Mi',
  },

  all(clusterNamespace='postgres', superUserPassword, replicationUserPassword): {
    operator: std.parseYaml(importstr './kubegres.yaml'),

    namespace: namespace.new(clusterNamespace),

    passwords: util.secretStringData('mypostgres-secret', clusterNamespace, {
      superUserPassword: superUserPassword,
      replicationUserPassword: replicationUserPassword,
    }),

    kubegres: {
      apiVersion: 'kubegres.reactive-tech.io/v1',
      kind: 'Kubegres',
      metadata: {
        name: $._config.kubegresName,
        namespace: clusterNamespace,
      },
      spec: {
        replicas: 2,
	image: '%s:%s' % [$._config.postgresImage.name, $._config.postgresImage.tag],
	database: {
	  size: $._config.databaseSize
	},
        env: [
          util.envValueFromSecret('POSTGRES_PASSWORD', $._config.passwordSecretName, 'superUserPassword'),
          util.envValueFromSecret('POSTGRES_REPLICATION_PASSWORD', $._config.passwordSecretName, 'replicationUserPassword'),
        ],
      },
    },
  },
}
