[working-directory: 'miniflux']
miniflux-takeoff:
	cargo build --target=wasm32-wasip1 --release
	yoke takeoff -namespace miniflux miniflux target/wasm32-wasip1/release/miniflux.wasm

[working-directory: 'miniflux']
miniflux-debug:
	cargo build --target=wasm32-wasip1
	yoke takeoff -stdout -namespace miniflux miniflux target/wasm32-wasip1/debug/miniflux.wasm

[working-directory: 'csi-driver-linode']
csi-driver-linode-debug:
	cargo build --target=wasm32-wasip1
	yoke takeoff -stdout -namespace kube-system csi-driver-linode target/wasm32-wasip1/debug/csi-driver-linode.wasm

[working-directory: 'csi-driver-linode']
csi-driver-linode-takeoff:
	cargo build --target=wasm32-wasip1 --release
	yoke takeoff -force-conflicts -namespace kube-system csi-driver-linode target/wasm32-wasip1/release/csi-driver-linode.wasm
