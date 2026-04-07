use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

fn nvc_bin() -> &'static str {
    env!("CARGO_BIN_EXE_nvc")
}

fn global_prefix_dir(base_dir: &Path) -> PathBuf {
    base_dir.join("global")
}

fn global_bin_dir(base_dir: &Path) -> PathBuf {
    if cfg!(windows) {
        global_prefix_dir(base_dir)
    } else {
        global_prefix_dir(base_dir).join("bin")
    }
}

fn node_bin_dir(base_dir: &Path, version: &str) -> PathBuf {
    let installation_dir = base_dir
        .join("node-versions")
        .join(version)
        .join("installation");
    if cfg!(windows) {
        installation_dir
    } else {
        installation_dir.join("bin")
    }
}

fn run_nvc(base_dir: &Path, args: &[&str]) -> Output {
    Command::new(nvc_bin())
        .args(args)
        .env("NVC_DIR", base_dir)
        .env("NO_COLOR", "1")
        .output()
        .expect("failed to run nvc")
}

fn run_nvc_ok(base_dir: &Path, args: &[&str]) -> Output {
    let output = run_nvc(base_dir, args);
    assert!(
        output.status.success(),
        "nvc {:?} failed\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
        args,
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn write_local_package(temp_dir: &TempDir) -> PathBuf {
    let package_dir = temp_dir.path().join("shared-package");
    let bin_dir = package_dir.join("bin");
    std::fs::create_dir_all(&bin_dir).unwrap();
    std::fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "nvc-shared-package",
  "version": "1.0.0",
  "bin": {
    "nvc-shared-test": "bin/cli.js"
  }
}"#,
    )
    .unwrap();
    let cli_path = bin_dir.join("cli.js");
    std::fs::write(&cli_path, "#!/usr/bin/env node\nconsole.log('shared-ok')\n").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = std::fs::metadata(&cli_path).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&cli_path, permissions).unwrap();
    }

    package_dir
}

#[test]
fn env_json_includes_shared_npm_prefix() {
    let base_dir = TempDir::new().unwrap();

    let output = run_nvc_ok(base_dir.path(), &["env", "--json"]);
    let payload: Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(
        payload["NPM_CONFIG_PREFIX"].as_str().unwrap(),
        global_prefix_dir(base_dir.path()).to_str().unwrap()
    );
    assert_eq!(
        payload["NVC_DIR"].as_str().unwrap(),
        base_dir.path().to_str().unwrap()
    );
}

#[test]
#[ignore = "real-download"]
fn exec_uses_shared_prefix_and_global_packages_are_shared() {
    let base_dir = TempDir::new().unwrap();
    let package_temp_dir = TempDir::new().unwrap();
    let package_dir = write_local_package(&package_temp_dir);

    run_nvc_ok(base_dir.path(), &["install", "12.0.0", "--progress=never"]);
    run_nvc_ok(base_dir.path(), &["install", "14.21.3", "--progress=never"]);

    let inspect_script = r#"const path = require("path"); console.log(JSON.stringify({ prefix: process.env.NPM_CONFIG_PREFIX, path: process.env.PATH.split(path.delimiter).slice(0, 3) }));"#;
    let inspect_output = run_nvc_ok(
        base_dir.path(),
        &["exec", "--using=12.0.0", "node", "-e", inspect_script],
    );
    let inspect: Value = serde_json::from_slice(&inspect_output.stdout).unwrap();

    assert_eq!(
        inspect["prefix"].as_str().unwrap(),
        global_prefix_dir(base_dir.path()).to_str().unwrap()
    );
    assert_eq!(
        inspect["path"][0].as_str().unwrap(),
        node_bin_dir(base_dir.path(), "v12.0.0").to_str().unwrap()
    );
    assert_eq!(
        inspect["path"][1].as_str().unwrap(),
        global_bin_dir(base_dir.path()).to_str().unwrap()
    );

    run_nvc_ok(
        base_dir.path(),
        &[
            "exec",
            "--using=12.0.0",
            "npm",
            "install",
            "-g",
            package_dir.to_str().unwrap(),
        ],
    );

    let prefix_output = run_nvc_ok(
        base_dir.path(),
        &["exec", "--using=14.21.3", "npm", "config", "get", "prefix"],
    );
    let prefix_path = PathBuf::from(String::from_utf8(prefix_output.stdout).unwrap().trim());
    assert_eq!(
        prefix_path.canonicalize().unwrap(),
        global_prefix_dir(base_dir.path()).canonicalize().unwrap()
    );

    let shared_bin_output = run_nvc_ok(
        base_dir.path(),
        &["exec", "--using=14.21.3", "nvc-shared-test"],
    );
    assert_eq!(
        String::from_utf8(shared_bin_output.stdout).unwrap().trim(),
        "shared-ok"
    );
}
