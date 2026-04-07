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
fn doctor_json_reports_warn_without_env() {
    let base_dir = TempDir::new().unwrap();

    let output = run_nvc_ok(base_dir.path(), &["doctor", "--json"]);
    let payload: Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(payload["status"].as_str().unwrap(), "warn");
    assert_eq!(
        payload["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|check| check["name"] == "multishell_path")
            .unwrap()["status"]
            .as_str()
            .unwrap(),
        "warn"
    );
}

#[test]
fn doctor_human_output_mentions_global_prefix() {
    let base_dir = TempDir::new().unwrap();

    let output = run_nvc_ok(base_dir.path(), &["doctor"]);
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("nvc doctor:"));
    assert!(stdout.contains("global_prefix"));
    assert!(stdout.contains(&base_dir.path().join("global").display().to_string()));
}
