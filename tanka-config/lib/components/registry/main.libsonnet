local k = import 'github.com/grafana/jsonnet-libs/ksonnet-util/kausal.libsonnet';
local util = import 'util/main.libsonnet';

local deployment = k.apps.v1.deployment;
local container = k.core.v1.container;
local containerPort = k.core.v1.containerPort;
local configMap = k.core.v1.configMap;
local service = k.core.v1.service;
local servicePort = k.core.v1.servicePort;
local volume = k.core.v1.volume;
local volumeMount = k.core.v1.volumeMount;

{
  _config:: {
    name: 'registry',
    image: {
      name: 'registry',
      tag: '2.8.3',
    },
    containerPort: 5000,
    servicePort: 80,
    hostname: 'registry.bgottlob.com',
    matchLabels: {
      app: 'registry',
    },
    authSecretName: 'registry-htpasswd-secret',
    s3SecretName: 'registry-s3-secret',
    configMapName: 'registry-config',
  },

  all(namespace='registry', s3Keys, authHtpasswd): {
    deployment:
      deployment.new(
        name=$._config.name,
        replicas=1,
        containers=[
          container.new(
            $._config.name,
            $._config.image.name + ':' + $._config.image.tag,
          )
          + container.withPorts([containerPort.new('app', $._config.containerPort)])
          + container.withEnv([
            util.envValueFromSecret('REGISTRY_STORAGE_S3_ACCESSKEY', $._config.s3SecretName, 'accesskey'),
            util.envValueFromSecret('REGISTRY_STORAGE_S3_SECRETKEY', $._config.s3SecretName, 'secretkey'),
          ])
          + container.withVolumeMounts([
            volumeMount.new('registry-config', '/etc/docker/registry', true),
            volumeMount.new('registry-htpasswd-secret', '/auth', true),
          ]),
        ],
      )
      + deployment.spec.template.spec.withVolumes([
        volume.fromConfigMap(
          'registry-config',
          $._config.configMapName,
          [{ key: 'config.yml', path: 'config.yml' }]
        ),
        volume.fromSecret('registry-htpasswd-secret', $._config.authSecretName),
      ])
      + deployment.metadata.withNamespace(namespace)
      + deployment.spec.selector.withMatchLabels($._config.matchLabels)
      + deployment.spec.template.metadata.withLabels($._config.matchLabels),

    configMap:
      configMap.new(
        $._config.configMapName,
        {
          'config.yml': |||
            version: 0.1
            log:
              fields:
                service: registry
              level: debug
            auth:
              htpasswd:
                realm: basic-realm
                path: /auth/htpasswd
            storage:
              cache:
                blobdescriptor: inmemory
              s3:
                region: us-east-1
                regionendpoint: us-east-1.linodeobjects.com/
                secure: true
                bucket: bgottlob-registry
            http:
              addr: :5000
              headers:
                X-Content-Type-Options: [nosniff]
              debug:
                addr: 127.0.0.1:5001
                prometheus:
                  enabled: false
                  path: /metrics
            health:
              storagedriver:
                enabled: true
            interval: 10s
            threshold: 3
          |||,
        }
      ) + configMap.metadata.withNamespace(namespace),

    s3Secret:
      util.secretStringData(
        name=$._config.s3SecretName,
        namespace=namespace,
        stringData={
          accesskey: s3Keys.accesskey,
          secretkey: s3Keys.secretkey,
        },
      ),

    authSecret:
      util.secretStringData(
        name=$._config.authSecretName,
        namespace=namespace,
        stringData={
          htpasswd: authHtpasswd,
        },
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
        servicePort=$._config.servicePort,
      ) + {
        ingress+: {
          metadata+: {
            annotations+: {
              // Allows images of any size to be uploaded
              'nginx.ingress.kubernetes.io/proxy-body-size': '0',
              'nginx.ingress.kubernetes.io/proxy-read-timeout': '6000',
              'nginx.ingress.kubernetes.io/proxy-send-timeout': '6000',
            },
          },
        },
      },
  },
}
