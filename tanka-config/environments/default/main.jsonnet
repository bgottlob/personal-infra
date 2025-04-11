// build with sops -d ../secrets.yaml | tk apply environments/secrets

local blog = import 'components/blog/main.libsonnet';
local certManager = import 'components/cert-manager/main.libsonnet';
local externalDNS = import 'components/external-dns/main.libsonnet';
local ingressNginx = import 'components/ingress-nginx/main.libsonnet';
local kubegres = import 'components/kubegres/main.libsonnet';
local kubePrometheusStack = import 'components/kube-prometheus-stack/main.libsonnet';
local miniflux = import 'components/miniflux/main.libsonnet';
local planka = import 'components/planka/main.libsonnet';
local registry = import 'components/registry/main.libsonnet';
local rmfakecloud = import 'components/rmfakecloud/main.libsonnet';
local velero = import 'components/velero/main.libsonnet';
local wallabag = import 'components/wallabag/main.libsonnet';

local secrets = std.parseYaml(importstr '/dev/stdin');

{
  local pgHost = 'main-postgres.postgres',
  local pgPort = 5432,
  local pgUrl = std.format(
    'postgres://%s:%s@' + pgHost,
    [
      secrets.postgres.super_user.username,
      secrets.postgres.super_user.password,
    ]
  ),

  local registryCreds = secrets.registry.login,

  blog: blog.all(registryCreds=registryCreds),

  certManager: certManager.all(
    clusterIssuer='letsencrypt-prod',
    ingressClass='nginx',
  ),

  externalDNS: externalDNS.all(
    linodeToken=secrets.linode.externalDNSToken,
  ),

  ingressNginx: ingressNginx.all(),

  kubegres: kubegres.all(
    superUserPassword=secrets.postgres.super_user.password,
    replicationUserPassword=secrets.postgres.replication.password
  ),

  kubePrometheusStack: kubePrometheusStack.all(),

  miniflux: miniflux.deployment() + miniflux.secrets(
    pgUrl=pgUrl,
    adminUsername=secrets.miniflux.admin.username,
    adminPassword=secrets.miniflux.admin.password,
  ),

  planka: planka.all(
    pgUrl=pgUrl,
    secretKey=secrets.planka.secret_key,
    adminName=secrets.planka.admin.name,
    adminEmail=secrets.planka.admin.email,
    adminUsername=secrets.planka.admin.username,
    adminPassword=secrets.planka.admin.password,
  ),

  registry: registry.all(
    s3Keys={
      accesskey: secrets.registry.bucket.access_key_id,
      secretkey: secrets.registry.bucket.secret_key,
    },
    authHtpasswd=secrets.registry.auth.htpasswd,
  ),

  rmfakecloud: rmfakecloud.all(),

  velero: velero.all(
    accessKeyId=secrets.velero.bucket.access_key_id,
    secretAccessKey=secrets.velero.bucket.secret_key,
  ),

  wallabag: wallabag.all(
    pgUser=secrets.postgres.super_user.username,
    pgPassword=secrets.postgres.super_user.password,
    pgHost=pgHost,
    pgPort=pgPort,
  ),
}
