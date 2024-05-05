local k = import 'github.com/grafana/jsonnet-libs/ksonnet-util/kausal.libsonnet';
local cm = import 'github.com/jsonnet-libs/cert-manager-libsonnet/1.10/main.libsonnet';
local util = import 'util/main.libsonnet';
local deployment = k.apps.v1.deployment;
local container = k.core.v1.container;
local containerPort = k.core.v1.containerPort;
local service = k.core.v1.service;
local servicePort = k.core.v1.servicePort;

{
  _config:: {
    name: 'miniflux',
    image: {
      name: 'miniflux/miniflux',
      tag: '2.0.41',
    },
    port: 8080,  // port the miniflux container listens on
    pgSecret: 'miniflux-postgres',
    adminSecret: 'miniflux-admin',
    hostname: 'miniflux.bgottlob.com',
  },

  deployment(namespace='miniflux'): {
    local matchLabels = {
      app: $._config.name,
    },

    local envValue(name, value) = {
      name: name,
      value: value,
    },

    local envValueFromSecret(name, secretName, secretKey) = {
      name: name,
      valueFrom: {
        secretKeyRef: {
          name: secretName,
          key: secretKey,
        },
      },
    },

    deployment: deployment.new(
                  name=$._config.name,
                  replicas=1,
                  containers=[
                    container.new(
                      $._config.name,
                      $._config.image.name + ':' + $._config.image.tag
                    )
                    + container.withPorts([containerPort.new('app', $._config.port)])
                    + container.withEnv([
                      envValueFromSecret('DATABASE_URL', $._config.pgSecret, 'database_url'),
                      envValue('RUN_MIGRATIONS', '1'),
                      envValue('CREATE_ADMIN', '1'),
                      envValueFromSecret('ADMIN_USERNAME', $._config.adminSecret, 'username'),
                      envValueFromSecret('ADMIN_PASSWORD', $._config.adminSecret, 'password'),
                    ]),
                  ],
                )
                + deployment.spec.selector.withMatchLabels(matchLabels)
                + deployment.spec.template.metadata.withLabels(matchLabels)
                + deployment.metadata.withNamespace(namespace),

    service: service.new(
      name=$._config.name,
      selector=matchLabels,
      ports=servicePort.new(port=80, targetPort=8080)
    ) + service.metadata.withNamespace(namespace),

    tlsIngress: util.tlsIngress(
      name=$._config.name,
      namespace=namespace,
      hostname=$._config.hostname
    ),
  },

  secrets(namespace='miniflux', pgUrl, adminUsername, adminPassword): {
    pgSecret: util.secretStringData(
      name=$._config.pgSecret,
      namespace=namespace,
      stringData={
        database_url: pgUrl + '/miniflux?sslmode=disable',
      }
    ),

    adminSecret: util.secretStringData(
      name=$._config.adminSecret,
      namespace=namespace,
      stringData={
        username: adminUsername,
        password: adminPassword,
      }
    ),
  },
}
