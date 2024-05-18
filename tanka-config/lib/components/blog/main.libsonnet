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
    name: 'blog',
    image: {
      name: 'registry.bgottlob.com/blog',
      tag: 'latest',
    },
    containerPort: 80,
    servicePort: 80,
    hostname: 'bgottlob.com',
    matchLabels: {
      app: 'blog',
    },
    imagePullSecretName: 'registry-creds',
  },

  all(namespace='blog', registryCreds): {
    deployment:
      deployment.new(
        name=$._config.name,
        replicas=1,
        containers=[
          container.new(
            $._config.name,
            $._config.image.name + ':' + $._config.image.tag,
          )
          + container.withImagePullPolicy('Always')
          + container.withPorts([containerPort.new('app', $._config.containerPort)]),
        ],
      )
      + deployment.spec.template.spec.withImagePullSecrets([{ name: $._config.imagePullSecretName }])
      + deployment.metadata.withNamespace(namespace)
      + deployment.spec.selector.withMatchLabels($._config.matchLabels)
      + deployment.spec.template.metadata.withLabels($._config.matchLabels),

    imagePullSecret:
      util.secretDockerRegistry(
        name=$._config.imagePullSecretName,
        namespace=namespace,
        creds=registryCreds,
      ),

    service:
      service.new(
        name=$._config.name,
        selector=$._config.matchLabels,
        ports=servicePort.new(port=$._config.servicePort, targetPort=$._config.containerPort)
      ) + service.metadata.withNamespace(namespace),

    tlsIngress:
      util.tlsIngress(
        name=$._config.name,
        namespace=namespace,
        hostname=$._config.hostname,
      ),
  },
}
