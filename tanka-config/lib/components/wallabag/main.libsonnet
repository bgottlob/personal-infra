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
    name: 'wallabag',
    image: {
      name: 'wallabag/wallabag',
      tag: '2.6.12',
    },
    port: 80, // port the wallabag container listens on
    servicePort: 80,
    hostname: 'wallabag.bgottlob.com',
    matchLabels: {
      app: 'wallabag'
    }
  },

  local pgSecret = $._config.name + '-postgres',

  all(namespace='wallabag', pgUser, pgPassword, pgHost, pgPort): {
    deployment: deployment.new(
      name = $._config.name,
      replicas = 1,
      containers = [
        container.new(
          $._config.name,
          $._config.image.name + ':' + $._config.image.tag,
        )
	+ container.withPorts([containerPort.new('app', $._config.port)])
	+ container.withEnv([
	  util.envValueFromSecret('POSTGRES_USER', pgSecret, 'user'),
	  util.envValueFromSecret('POSTGRES_PASSWORD', pgSecret, 'password'),
	  util.envValue('SYMFONY__ENV__DATABASE_DRIVER', 'pdo_pgsql'),
	  util.envValueFromSecret('SYMFONY__ENV__DATABASE_HOST', pgSecret, 'host'),
	  util.envValueFromSecret('SYMFONY__ENV__DATABASE_PORT', pgSecret, 'port'),
	  util.envValue('SYMFONY__ENV__DATABASE_NAME', $._config.name),
	  util.envValueFromSecret('SYMFONY__ENV__DATABASE_USER', pgSecret, 'user'),
	  util.envValueFromSecret('SYMFONY__ENV__DATABASE_PASSWORD', pgSecret, 'password'),
	  util.envValue('SYMFONY__ENV__DOMAIN_NAME', 'https://' + $._config.hostname),
	  util.envValue('SYMFONY__ENV__DATABASE_DRIVER', 'pdo_pgsql'),
	  util.envValue('SYMFONY__ENV__SERVER_NAME', "Brandon's Wallabag"),
	  util.envValue('SYMFONY__ENV__FOSUSER_REGISTRATION', 'false'),
	])
      ]
    )
    + deployment.metadata.withNamespace(namespace)
    + deployment.spec.selector.withMatchLabels($._config.matchLabels)
    + deployment.spec.template.metadata.withLabels($._config.matchLabels),

    secret: util.secretStringData(pgSecret, namespace, {
	user: pgUser,
	password: pgPassword,
	host: pgHost,
	port: std.toString(pgPort),
    }),

    service: service.new(
      name=$._config.name,
      selector=$._config.matchLabels,
      ports=servicePort.new(port=$._config.servicePort, targetPort=$._config.port)
    ) + service.metadata.withNamespace(namespace),

    tlsIngress: util.tlsIngress(
      name=$._config.name,
      namespace=namespace,
      hostname=$._config.hostname,
      servicePort=$._config.servicePort,
    ),
  }
}
