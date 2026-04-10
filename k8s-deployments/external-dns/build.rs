fn main() {
    let secrets_path = std::env::var("SECRETS_FILE").unwrap_or("../../secrets.yaml".into());
    let secrets = secrets::decrypt_parse_secrets(&secrets_path)
        .unwrap_or_else(|_| panic!("Should parse {} file into Secrets struct", secrets_path));

    println!(
        "cargo:rustc-env=LINODE_TOKEN={}",
        secrets.linode.external_dns_token,
    );

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", secrets_path);
}
