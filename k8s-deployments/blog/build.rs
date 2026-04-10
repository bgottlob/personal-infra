use secrets::decrypt_parse_secrets;

fn main() {
    let secrets_path = std::env::var("SECRETS_FILE").unwrap_or("../../secrets.yaml".into());
    let secrets = decrypt_parse_secrets(&secrets_path)
        .unwrap_or_else(|_| panic!("Should parse {} file into Secrets struct", secrets_path));

    println!(
        "cargo:rustc-env=REGISTRY_SERVER={}",
        secrets.registry.login.server
    );
    println!(
        "cargo:rustc-env=REGISTRY_USERNAME={}",
        secrets.registry.login.username
    );
    println!(
        "cargo:rustc-env=REGISTRY_PASSWORD={}",
        secrets.registry.login.password
    );
    println!(
        "cargo:rustc-env=REGISTRY_EMAIL={}",
        secrets.registry.login.email
    );

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", secrets_path);
}
