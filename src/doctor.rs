use crate::config::NvcConfig;
use crate::current_version;
use crate::shell::{infer_shell, maybe_fix_windows_path};
use serde::Serialize;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Ok,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct Check {
    pub name: &'static str,
    pub status: CheckStatus,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub status: CheckStatus,
    pub checks: Vec<Check>,
}

fn contains_path_entry(path_var: &OsStr, expected_path: &Path) -> bool {
    let expected_str = expected_path.to_str();
    let fixed_expected = expected_str.and_then(maybe_fix_windows_path);
    let fixed_expected = fixed_expected.as_deref();

    std::env::split_paths(path_var).any(|path| {
        path == expected_path || fixed_expected.is_some_and(|fixed| path.to_str() == Some(fixed))
    })
}

fn check_base_dir(config: &NvcConfig) -> Check {
    let base_dir = config.base_dir_with_default();
    let status = if base_dir.exists() {
        CheckStatus::Ok
    } else {
        CheckStatus::Warn
    };
    let detail = format!("base directory: {}", base_dir.display());

    Check {
        name: "base_dir",
        status,
        detail,
    }
}

fn try_write_probe(dir: &Path) -> bool {
    if !dir.exists() {
        return false;
    }

    let probe_path = dir.join(format!(
        ".nvc-doctor-probe-{}-{}",
        std::process::id(),
        chrono::Utc::now().timestamp_millis()
    ));

    match std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&probe_path)
    {
        Ok(_) => {
            let _ = std::fs::remove_file(probe_path);
            true
        }
        Err(_) => false,
    }
}

fn check_global_prefix(config: &NvcConfig) -> Check {
    let prefix = config.global_prefix_dir();
    if !prefix.exists() {
        return Check {
            name: "global_prefix",
            status: CheckStatus::Warn,
            detail: format!(
                "shared global prefix does not exist yet: {}",
                prefix.display()
            ),
        };
    }

    let writable = try_write_probe(&prefix);
    let status = if writable {
        CheckStatus::Ok
    } else {
        CheckStatus::Error
    };
    let detail = format!(
        "shared global prefix: {} ({})",
        prefix.display(),
        if writable { "writable" } else { "not writable" }
    );

    Check {
        name: "global_prefix",
        status,
        detail,
    }
}

fn check_shell() -> Check {
    match infer_shell() {
        Some(shell) => Check {
            name: "shell",
            status: CheckStatus::Ok,
            detail: format!("detected shell: {shell:?}"),
        },
        None => Check {
            name: "shell",
            status: CheckStatus::Warn,
            detail: "could not infer shell from process tree".to_string(),
        },
    }
}

fn check_multishell(config: &NvcConfig) -> Check {
    match config.multishell_path() {
        Some(path) if path.exists() => Check {
            name: "multishell_path",
            status: CheckStatus::Ok,
            detail: format!("active multishell path: {}", path.display()),
        },
        Some(path) => Check {
            name: "multishell_path",
            status: CheckStatus::Warn,
            detail: format!(
                "NVC_MULTISHELL_PATH is set but missing on disk: {}",
                path.display()
            ),
        },
        None => Check {
            name: "multishell_path",
            status: CheckStatus::Warn,
            detail: "NVC_MULTISHELL_PATH is not set; `nvc env` does not appear to be active"
                .to_string(),
        },
    }
}

fn node_bin_for_multishell(path: &Path) -> PathBuf {
    if cfg!(windows) {
        path.to_path_buf()
    } else {
        path.join("bin")
    }
}

fn check_node_bin_on_path(config: &NvcConfig) -> Check {
    let Some(multishell_path) = config.multishell_path() else {
        return Check {
            name: "node_bin_on_path",
            status: CheckStatus::Warn,
            detail: "skipped because NVC_MULTISHELL_PATH is not set".to_string(),
        };
    };

    let Some(path_var) = std::env::var_os("PATH") else {
        return Check {
            name: "node_bin_on_path",
            status: CheckStatus::Error,
            detail: "PATH is not set".to_string(),
        };
    };

    let node_bin = node_bin_for_multishell(multishell_path);
    let on_path = contains_path_entry(&path_var, &node_bin);

    Check {
        name: "node_bin_on_path",
        status: if on_path {
            CheckStatus::Ok
        } else {
            CheckStatus::Warn
        },
        detail: format!(
            "active node bin {} {} on PATH",
            node_bin.display(),
            if on_path { "is" } else { "is not" }
        ),
    }
}

fn check_global_bin_on_path(config: &NvcConfig) -> Check {
    let Some(path_var) = std::env::var_os("PATH") else {
        return Check {
            name: "global_bin_on_path",
            status: CheckStatus::Error,
            detail: "PATH is not set".to_string(),
        };
    };

    let global_bin = config.global_bin_dir();
    let on_path = contains_path_entry(&path_var, &global_bin);

    Check {
        name: "global_bin_on_path",
        status: if on_path {
            CheckStatus::Ok
        } else {
            CheckStatus::Warn
        },
        detail: format!(
            "shared global bin {} {} on PATH",
            global_bin.display(),
            if on_path { "is" } else { "is not" }
        ),
    }
}

fn check_current_version(config: &NvcConfig) -> Check {
    match current_version::current_version(config) {
        Ok(Some(version)) => Check {
            name: "current_version",
            status: CheckStatus::Ok,
            detail: format!("current version: {}", version.v_str()),
        },
        Ok(None) => Check {
            name: "current_version",
            status: CheckStatus::Warn,
            detail: "no active version resolved from multishell state".to_string(),
        },
        Err(error) => Check {
            name: "current_version",
            status: CheckStatus::Warn,
            detail: format!("could not resolve current version: {error}"),
        },
    }
}

pub fn build_report(config: &NvcConfig) -> Report {
    let checks = vec![
        check_base_dir(config),
        check_global_prefix(config),
        check_shell(),
        check_multishell(config),
        check_node_bin_on_path(config),
        check_global_bin_on_path(config),
        check_current_version(config),
    ];

    let status = checks
        .iter()
        .map(|check| check.status)
        .max()
        .unwrap_or(CheckStatus::Ok);

    Report { status, checks }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_without_multishell_is_warn() {
        let config =
            NvcConfig::default().with_base_dir(Some(std::env::temp_dir().join("nvc-doctor-test")));

        let report = build_report(&config);

        assert_eq!(report.status, CheckStatus::Warn);
        assert!(report
            .checks
            .iter()
            .any(|check| { check.name == "multishell_path" && check.status == CheckStatus::Warn }));
    }
}
