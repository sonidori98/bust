use clap::Parser;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
#[command(name = "xtask", about = "bust build tool")]
enum Xtask {
    Build {},
    Install {
        #[arg(long, default_value = "/usr/local")]
        prefix: String,
        #[arg(long)]
        bindir: Option<String>,
        #[arg(long)]
        libdir: Option<String>,
    },
    Uninstall {
        #[arg(long, default_value = "/usr/local")]
        prefix: String,
        #[arg(long)]
        bindir: Option<String>,
        #[arg(long)]
        libdir: Option<String>,
    },
    Clean {},
    Test {},
}

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn cargo() -> Command {
    let root = project_root();
    let mut cmd = Command::new("cargo");
    cmd.current_dir(&root);
    cmd
}

fn build_bust() {
    println!(">> Building bust ...");
    let status = cargo()
        .args(["build", "--release", "-p", "bust"])
        .status()
        .expect("Failed to run cargo");
    assert!(status.success(), "bust build failed");
}

fn build_libb() {
    println!(">> Building libb ...");
    let status = cargo()
        .args(["build", "--release", "-p", "libb"])
        .status()
        .expect("Failed to run cargo");
    assert!(status.success(), "libb build failed");
}

fn ensure_dir(dir: &Path) {
    if let Err(e) = std::fs::create_dir_all(dir) {
        if e.raw_os_error() == Some(13) {
            println!("   (elevating via sudo for mkdir)");
            let status = Command::new("sudo")
                .args(["mkdir", "-p"])
                .arg(dir)
                .status()
                .expect("Failed to run sudo mkdir");
            assert!(status.success(), "sudo mkdir failed");
        } else {
            panic!("Failed to create directory {}: {}", dir.display(), e);
        }
    }
}

fn copy_file(src: &Path, dst: &Path) {
    if let Err(e) = std::fs::copy(src, dst) {
        if e.raw_os_error() == Some(13) {
            println!("   (elevating via sudo for cp)");
            let status = Command::new("sudo")
                .args(["cp"])
                .arg(src)
                .arg(dst)
                .status()
                .expect("Failed to run sudo cp");
            assert!(status.success(), "sudo cp failed");
        } else {
            panic!("Failed to copy {} -> {}: {}", src.display(), dst.display(), e);
        }
    }
}

fn set_perms(path: &Path, mode: u32) {
    if let Err(e) = std::fs::set_permissions(path, PermissionsExt::from_mode(mode)) {
        if e.raw_os_error() == Some(1) {
            // EPERM on chmod after sudo cp — ok, install already set mode
        } else {
            panic!("Failed to set permissions on {}: {}", path.display(), e);
        }
    }
}

fn install_file(src: &Path, dst: &Path, mode: u32) {
    if let Some(parent) = dst.parent() {
        ensure_dir(parent);
    }
    copy_file(src, dst);
    set_perms(dst, mode);
    println!(
        "   installed {} -> {} ({:o})",
        src.display(),
        dst.display(),
        mode
    );
}

fn cmd_install(prefix: &str, bindir: Option<&str>, libdir: Option<&str>) {
    let root = project_root();
    let bindir = bindir
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{}/bin", prefix));
    let libdir = libdir
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{}/lib64", prefix));

    build_bust();
    build_libb();

    install_file(
        &root.join("target/release/bust"),
        &Path::new(&bindir).join("bust"),
        0o755,
    );
    install_file(
        &root.join("target/release/liblibb.a"),
        &Path::new(&libdir).join("liblibb.a"),
        0o644,
    );
}

fn cmd_uninstall(prefix: &str, bindir: Option<&str>, libdir: Option<&str>) {
    let bindir = bindir
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{}/bin", prefix));
    let libdir = libdir
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{}/lib64", prefix));

    for path in [format!("{}/bust", bindir), format!("{}/liblibb.a", libdir)] {
        let p = Path::new(&path);
        if p.exists() {
            if let Err(e) = std::fs::remove_file(p) {
                if e.raw_os_error() == Some(13) {
                    println!("   (elevating via sudo for rm)");
                    let status = Command::new("sudo")
                        .args(["rm", "-f"])
                        .arg(&path)
                        .status()
                        .expect("Failed to run sudo rm");
                    assert!(status.success(), "sudo rm failed");
                } else {
                    panic!("Failed to remove {}: {}", path, e);
                }
            }
            println!("   removed {}", path);
        }
    }
}

fn main() {
    match Xtask::parse() {
        Xtask::Build {} => {
            build_bust();
            build_libb();
        }
        Xtask::Install { prefix, bindir, libdir } => {
            cmd_install(&prefix, bindir.as_deref(), libdir.as_deref());
        }
        Xtask::Uninstall { prefix, bindir, libdir } => {
            cmd_uninstall(&prefix, bindir.as_deref(), libdir.as_deref());
        }
        Xtask::Clean {} => {
            println!(">> cargo clean ...");
            let status = cargo().arg("clean").status().expect("Failed to run cargo");
            assert!(status.success());
        }
        Xtask::Test {} => {
            println!(">> cargo test ...");
            let status = cargo().arg("test").status().expect("Failed to run cargo");
            assert!(status.success());
        }
    }
}
