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

[working-directory: 'cert-manager']
cert-manager-debug:
	cargo build --target=wasm32-wasip1
	yoke takeoff -cross-namespace -stdout -namespace cert-manager cert-manager target/wasm32-wasip1/debug/cert-manager.wasm

[working-directory: 'cert-manager']
cert-manager-takeoff:
	cargo build --target=wasm32-wasip1 --release
	yoke takeoff -cross-namespace -namespace cert-manager cert-manager target/wasm32-wasip1/release/cert-manager.wasm

[working-directory: 'cnpg-database']
cnpg-database-debug:
	cargo build --target=wasm32-wasip1
	yoke takeoff -stdout -namespace main-db main-db target/wasm32-wasip1/debug/cnpg-database.wasm

[working-directory: 'cnpg-database']
cnpg-database-takeoff:
	cargo build --target=wasm32-wasip1 --release
	yoke takeoff -force-conflicts -create-namespace -namespace main-db main-db target/wasm32-wasip1/release/cnpg-database.wasm

[working-directory: 'cnpg-database']
cnpg-database-diff:
	cargo build --target=wasm32-wasip1 --release
	yoke takeoff -diff-only -create-namespace -namespace main-db main-db target/wasm32-wasip1/release/cnpg-database.wasm
