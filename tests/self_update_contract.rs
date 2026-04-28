use std::process::{Command, Output};

fn nvc_bin() -> &'static str {
    env!("CARGO_BIN_EXE_nvc")
}

fn run_nvc(args: &[&str]) -> Output {
    Command::new(nvc_bin())
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .expect("failed to run nvc")
}

#[test]
fn self_update_help_exposes_version_option() {
    let output = run_nvc(&["self", "update", "--help"]);

    assert!(
        output.status.success(),
        "nvc self update --help failed\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("--version"));
}
