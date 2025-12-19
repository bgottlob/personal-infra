use secrets::decrypt_parse_secrets;

fn main() {
    let secrets_path = std::env::var("SECRETS_FILE").unwrap_or("../secrets.yaml".into());
    let secrets = decrypt_parse_secrets(&secrets_path)
        .unwrap_or_else(|_| panic!("Should parse {} file into Secrets struct", secrets_path));

    println!(
        "cargo:rustc-env=S3_ACCESS_KEY={}",
        secrets.registry.bucket.access_key_id
    );
    println!(
        "cargo:rustc-env=S3_SECRET_KEY={}",
        secrets.registry.bucket.secret_key
    );
    println!(
        "cargo:rustc-env=AUTH_HTPASSWD={}",
        secrets.registry.auth.htpasswd,
    );

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", secrets_path);
}

