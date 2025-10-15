use secrets::decrypt_parse_secrets;

fn main() {
    let secrets_path = std::env::var("SECRETS_FILE").unwrap_or("../secrets.yaml".into());
    let secrets = decrypt_parse_secrets(&secrets_path)
        .unwrap_or_else(|_| panic!("Should parse {} file into Secrets struct", secrets_path));

    println!(
        "cargo:rustc-env=POSTGRES_USERNAME={}",
        secrets.postgres.app_user.username
    );
    println!(
        "cargo:rustc-env=POSTGRES_PASSWORD={}",
        secrets.postgres.app_user.password
    );
    println!(
        "cargo:rustc-env=MINIFLUX_ADMIN_USERNAME={}",
        secrets.miniflux.admin.username
    );
    println!(
        "cargo:rustc-env=MINIFLUX_ADMIN_PASSWORD={}",
        secrets.miniflux.admin.password
    );

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", secrets_path);
}
