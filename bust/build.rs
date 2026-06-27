fn main() {
    let out_dir = std::path::PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").unwrap(),
    );
    // workspace root = bust/Cargo.toml's parent
    let workspace_root = out_dir.parent().unwrap();
    let target_dir = std::path::PathBuf::from(
        std::env::var("CARGO_TARGET_DIR")
            .unwrap_or_else(|_| format!("{}/target", workspace_root.display())),
    );
    let profile = std::env::var("PROFILE").unwrap();
    let libb_path = target_dir.join(&profile).join("liblibb.a");
    println!("cargo:rustc-env=LIBB_PATH={}", libb_path.display());
}
