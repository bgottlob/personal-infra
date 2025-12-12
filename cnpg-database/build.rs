fn main() {
    let secrets_path = std::env::var("SECRETS_FILE").unwrap_or("../secrets.yaml".into());
    let secrets = secrets::decrypt_parse_secrets(&secrets_path)
        .unwrap_or_else(|_| panic!("Should parse {} file into Secrets struct", secrets_path));

    println!(
        "cargo:rustc-env=APP_USER_USERNAME={}",
        secrets.postgres.app_user.username
    );
    println!(
        "cargo:rustc-env=APP_USER_PASSWORD={}",
        secrets.postgres.app_user.password
    );

    println!(
        "cargo:rustc-env=SUPER_USER_USERNAME={}",
        secrets.postgres.super_user.username
    );
    println!(
        "cargo:rustc-env=SUPER_USER_PASSWORD={}",
        secrets.postgres.super_user.password
    );

    println!(
        "cargo:rustc-env=ACCESS_KEY_ID={}",
        secrets.postgres.backup_bucket.access_key_id
    );
    println!(
        "cargo:rustc-env=SECRET_KEY={}",
        secrets.postgres.backup_bucket.secret_key
    );

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", secrets_path);
}
