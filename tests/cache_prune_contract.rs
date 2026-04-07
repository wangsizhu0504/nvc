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

fn downloads_dir(base_dir: &Path) -> std::path::PathBuf {
    base_dir.join("node-versions").join(".downloads")
}

#[test]
fn cache_dir_prints_downloads_directory() {
    let base_dir = TempDir::new().unwrap();
    let output = run_nvc_ok(base_dir.path(), &["cache", "dir"]);

    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        downloads_dir(base_dir.path()).to_str().unwrap()
    );
}

#[test]
fn cache_size_and_clear_manage_download_cache() {
    let base_dir = TempDir::new().unwrap();
    let downloads_dir = downloads_dir(base_dir.path());
    std::fs::create_dir_all(&downloads_dir).unwrap();
    std::fs::write(downloads_dir.join("artifact.bin"), b"hello world").unwrap();

    let size_output = run_nvc_ok(base_dir.path(), &["cache", "size", "--bytes"]);
    assert_eq!(String::from_utf8(size_output.stdout).unwrap().trim(), "11");

    run_nvc_ok(base_dir.path(), &["cache", "clear"]);
    assert!(!downloads_dir.join("artifact.bin").exists());
}

#[test]
fn prune_downloads_supports_dry_run() {
    let base_dir = TempDir::new().unwrap();
    let downloads_dir = downloads_dir(base_dir.path());
    std::fs::create_dir_all(&downloads_dir).unwrap();
    std::fs::write(downloads_dir.join("artifact.bin"), b"hello").unwrap();

    run_nvc_ok(base_dir.path(), &["prune", "--downloads", "--dry-run"]);
    assert!(downloads_dir.join("artifact.bin").exists());

    run_nvc_ok(base_dir.path(), &["prune", "--downloads"]);
    assert!(!downloads_dir.join("artifact.bin").exists());
}

#[test]
#[cfg(unix)]
fn prune_aliases_removes_broken_aliases() {
    let base_dir = TempDir::new().unwrap();
    let aliases_dir = base_dir.path().join("aliases");
    std::fs::create_dir_all(&aliases_dir).unwrap();
    std::os::unix::fs::symlink("/definitely/missing/path", aliases_dir.join("broken")).unwrap();

    run_nvc_ok(base_dir.path(), &["prune", "--aliases"]);
    assert!(!aliases_dir.join("broken").exists());
}
