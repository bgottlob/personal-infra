// build with sops -d ../secrets.yaml | tk apply environments/secrets

local miniflux = import 'components/miniflux/main.libsonnet';
local planka = import 'components/planka/main.libsonnet';
local rmfakecloud = import 'components/rmfakecloud/main.libsonnet';
local wallabag = import 'components/wallabag/main.libsonnet';

local secrets = std.parseYaml(importstr '/dev/stdin');

{
  local pgHost = 'mypostgres.postgres',
  local pgPort = 5432,
  local pgUrl = std.format(
    'postgres://%s:%s@mypostgres.postgres',
    [
      secrets.postgres.super_user.username,
      secrets.postgres.super_user.password,
    ]
  ),

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

  rmfakecloud: rmfakecloud.all(),

  wallabag: wallabag.all(
    pgUser=secrets.postgres.super_user.username,
    pgPassword=secrets.postgres.super_user.password,
    pgHost=pgHost,
    pgPort=pgPort,
  ),
}
