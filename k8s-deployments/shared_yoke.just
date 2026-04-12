# Build and deploy to the cluster
takeoff extra_flags="":
	cargo build --target=wasm32-wasip1 --release
	yoke takeoff {{default_flags}} {{extra_flags}} {{name}} {{source_directory()}}/target/wasm32-wasip1/release/{{name}}.wasm

# Build and print rendered manifests to stdout without deploying
debug extra_flags="":
	cargo build --target=wasm32-wasip1
	yoke takeoff -stdout {{default_flags}} {{extra_flags}} {{name}} {{source_directory()}}/target/wasm32-wasip1/debug/{{name}}.wasm

# Build and show a diff of changes without deploying
diff extra_flags="":
	cargo build --target=wasm32-wasip1
	yoke takeoff -diff-only {{default_flags}} {{extra_flags}} {{name}} {{source_directory()}}/target/wasm32-wasip1/debug/{{name}}.wasm
