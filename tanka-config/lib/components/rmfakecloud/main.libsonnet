local k = import 'github.com/grafana/jsonnet-libs/ksonnet-util/kausal.libsonnet';
local cm = import 'github.com/jsonnet-libs/cert-manager-libsonnet/1.10/main.libsonnet';
local util = import 'util/main.libsonnet';

local statefulSet = k.apps.v1.statefulSet;
local container = k.core.v1.container;
local containerPort = k.core.v1.containerPort;
local service = k.core.v1.service;
local servicePort = k.core.v1.servicePort;
local volumeClaimTemplate = k.core.v1.ephemeralVolumeSource.volumeClaimTemplate;

{
  _config:: {
    name: 'rmfakecloud',
    image: {
      name: 'ddvk/rmfakecloud',
      tag: '0.0.16',
    },
    containerPort: 3000,
    servicePort: 80,
    hostname: 'remarkable.bgottlob.com',
    matchLabels: {
      app: 'rmfakecloud',
    },
    maxUploadSize: '300m',
  },

  all(namespace='rmfakecloud'): {
    statefulSet: statefulSet.new(
                   name=$._config.name,
                   replicas=1,
                   containers=[
                     container.new(
                       $._config.name,
                       $._config.image.name + ':' + $._config.image.tag,
                     )
                     + container.withPorts([containerPort.new('app', $._config.containerPort)])
                     + container.withEnv([
                       util.envValue('DATADIR', '/data/rmfakecloud'),
                       util.envValue('STORAGE_URL', 'https://' + $._config.hostname),
                       util.envValue('PORT', std.toString($._config.containerPort)),
                     ])
                     + container.withVolumeMounts([
                       {
                         name: 'data',
                         mountPath: '/data/rmfakecloud',
                       },
                     ]),
                   ],
                   volumeClaims=[{
                     metadata: {
                       name: 'data',
                     },
                     spec: {
                       accessModes: ['ReadWriteOncePod'],
                       storageClassName: 'linode-block-storage-retain',
                       resources: {
                         requests: {
                           storage: '15Gi',
                         },
                       },
                     },
                   }]
                 ) + statefulSet.metadata.withNamespace(namespace)
                 + statefulSet.spec.selector.withMatchLabels($._config.matchLabels)
                 + statefulSet.spec.template.metadata.withLabels($._config.matchLabels),

    service: service.new(
      name=$._config.name,
      selector=$._config.matchLabels,
      ports=servicePort.new(port=$._config.servicePort, targetPort=$._config.containerPort)
    ) + service.metadata.withNamespace(namespace),

    tlsIngress: util.tlsIngress(
      name=$._config.name,
      namespace=namespace,
      hostname=$._config.hostname,
      servicePort=$._config.servicePort,
    ) + {
      ingress+: {
        metadata+: {
          annotations+:
            {
              'nginx.ingress.kubernetes.io/proxy-body-size': $._config.maxUploadSize,
            },
        },
      },
    },
  },
}
