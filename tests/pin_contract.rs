use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

fn nvc_bin() -> &'static str {
    env!("CARGO_BIN_EXE_nvc")
}

fn run_nvc(base_dir: &Path, current_dir: &Path, args: &[&str]) -> Output {
    Command::new(nvc_bin())
        .args(args)
        .current_dir(current_dir)
        .env("NVC_DIR", base_dir)
        .env("NO_COLOR", "1")
        .output()
        .expect("failed to run nvc")
}

fn run_nvc_ok(base_dir: &Path, current_dir: &Path, args: &[&str]) -> Output {
    let output = run_nvc(base_dir, current_dir, args);
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
fn pin_writes_explicit_version_to_node_version() {
    let base_dir = TempDir::new().unwrap();
    let project_dir = TempDir::new().unwrap();

    run_nvc_ok(base_dir.path(), project_dir.path(), &["pin", "22"]);

    assert_eq!(
        std::fs::read_to_string(project_dir.path().join(".node-version")).unwrap(),
        "22\n"
    );
}

#[test]
#[cfg(unix)]
fn pin_without_version_writes_current_version() {
    let base_dir = TempDir::new().unwrap();
    let project_dir = TempDir::new().unwrap();
    let installation = base_dir
        .path()
        .join("node-versions")
        .join("v20.11.1")
        .join("installation");
    let multishell_path = base_dir.path().join("multishell");
    std::fs::create_dir_all(&installation).unwrap();
    std::os::unix::fs::symlink(&installation, &multishell_path).unwrap();

    let output = Command::new(nvc_bin())
        .arg("pin")
        .current_dir(project_dir.path())
        .env("NVC_DIR", base_dir.path())
        .env("NVC_MULTISHELL_PATH", &multishell_path)
        .env("NO_COLOR", "1")
        .output()
        .expect("failed to run nvc");

    assert!(
        output.status.success(),
        "nvc pin failed\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(project_dir.path().join(".node-version")).unwrap(),
        "v20.11.1\n"
    );
}

#[test]
fn pin_rejects_multiline_explicit_version() {
    let base_dir = TempDir::new().unwrap();
    let project_dir = TempDir::new().unwrap();

    let output = run_nvc(base_dir.path(), project_dir.path(), &["pin", "20\n22"]);

    assert!(!output.status.success());
    assert!(!project_dir.path().join(".node-version").exists());
}
