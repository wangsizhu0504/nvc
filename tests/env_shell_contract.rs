use serde_json::Value;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

fn nvc_bin() -> &'static str {
    env!("CARGO_BIN_EXE_nvc")
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

#[test]
fn env_json_contract_exposes_shared_prefix_and_nvc_dir() {
    let base_dir = TempDir::new().unwrap();
    let output = run_nvc_ok(base_dir.path(), &["env", "--json"]);
    let payload: Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(
        payload["NVC_DIR"].as_str().unwrap(),
        base_dir.path().to_str().unwrap()
    );
    assert_eq!(
        payload["NPM_CONFIG_PREFIX"].as_str().unwrap(),
        base_dir.path().join("global").to_str().unwrap()
    );
}

#[cfg(not(windows))]
#[test]
fn bash_use_on_cd_output_contains_shared_prefix_and_autoload_hook() {
    let base_dir = TempDir::new().unwrap();
    let output = run_nvc_ok(base_dir.path(), &["env", "--shell", "bash", "--use-on-cd"]);
    let shell_script = String::from_utf8(output.stdout).unwrap();
    let expected_global_bin = if cfg!(windows) {
        base_dir.path().join("global")
    } else {
        base_dir.path().join("global").join("bin")
    };

    assert!(shell_script.contains("export NPM_CONFIG_PREFIX="));
    assert!(shell_script.contains(&expected_global_bin.display().to_string()));
    assert!(shell_script.contains("__nvc_use_if_file_found()"));
    assert!(shell_script.contains("nvc use --silent-if-unchanged"));
}
