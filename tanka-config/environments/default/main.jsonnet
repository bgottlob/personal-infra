// build with sops -d ../secrets.yaml | tk apply environments/secrets

local blog = import 'components/blog/main.libsonnet';
local certManager = import 'components/cert-manager/main.libsonnet';
local cloudnativePg = import 'components/cloudnative-pg/main.libsonnet';
local externalDNS = import 'components/external-dns/main.libsonnet';
local ingressNginx = import 'components/ingress-nginx/main.libsonnet';
local kubePrometheusStack = import 'components/kube-prometheus-stack/main.libsonnet';
local miniflux = import 'components/miniflux/main.libsonnet';
local planka = import 'components/planka/main.libsonnet';
local registry = import 'components/registry/main.libsonnet';
local rmfakecloud = import 'components/rmfakecloud/main.libsonnet';
local velero = import 'components/velero/main.libsonnet';
local wallabag = import 'components/wallabag/main.libsonnet';

local secrets = std.parseYaml(importstr '/dev/stdin');

{
  local pgHost = 'cnpg-database-cluster-rw.cnpg-database',
  local pgPort = 5432,
  local pgUrl = std.format(
    'postgres://%s:%s@' + pgHost,
    [
      secrets.postgres.app_user.username,
      secrets.postgres.app_user.password,
    ]
  ),

  local registryCreds = secrets.registry.login,

  blog: blog.all(registryCreds=registryCreds),

  certManager: certManager.all(
    clusterIssuer='letsencrypt-prod',
    ingressClass='nginx',
  ),

  cloudnativePg: cloudnativePg.all(
    backupBucketAccessKey=secrets.postgres.backup_bucket.access_key_id,
    backupBucketSecretKey=secrets.postgres.backup_bucket.secret_key,
    databases=[
      'miniflux',
      'planka',
      'wallabag',
    ]
  ),

  externalDNS: externalDNS.all(
    linodeToken=secrets.linode.externalDNSToken,
  ),

  ingressNginx: ingressNginx.all(),

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
    pgUser=secrets.postgres.app_user.username,
    pgPassword=secrets.postgres.app_user.password,
    pgHost=pgHost,
    pgPort=pgPort,
  ),
}
