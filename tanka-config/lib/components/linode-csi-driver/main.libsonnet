local util = import 'util/main.libsonnet';

{
  all(namespace='kube-system', token): {
    // Get this file by downloading from these instructions:
    // https://www.linode.com/docs/guides/install-the-linode-block-storage-csi-driver-on-unmanaged-kubernetes/#apply-csi-driver-to-your-cluster

    driver: std.parseYaml(importstr './linode-blockstorage-csi-driver.yaml'),
    secret: util.secretStringData('linode', namespace, {
      region: 'us-east',
      token: token,
    }),
  },
}
