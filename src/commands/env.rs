use super::command::Command;
use super::r#use::Use;
use crate::config::NvcConfig;
use crate::fs::symlink_dir;
use crate::outln;
use crate::shell::{infer_shell, Shell, Shells};
use clap::ValueEnum;
use colored::Colorize;
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::IsTerminal;
use thiserror::Error;

#[derive(clap::Parser, Debug, Default)]
pub struct Env {
    /// The shell syntax to use. Infers when missing.
    #[clap(long)]
    shell: Option<Shells>,
    /// Print JSON instead of shell commands.
    #[clap(long, conflicts_with = "shell")]
    json: bool,
    /// Deprecated. This is the default now.
    #[clap(long, hide = true)]
    multi: bool,
    /// Print the script to change Node versions every directory change
    #[clap(long)]
    use_on_cd: bool,
}

fn generate_symlink_path() -> String {
    format!(
        "{}_{}",
        std::process::id(),
        chrono::Utc::now().timestamp_millis(),
    )
}

fn make_symlink(config: &NvcConfig) -> Result<std::path::PathBuf, Error> {
    let base_dir = config.multishell_storage();
    std::fs::create_dir_all(&base_dir)?;
    std::fs::create_dir_all(config.aliases_dir())?;

    #[cfg(windows)]
    if !config.default_version_dir().exists() {
        std::fs::create_dir_all(config.default_version_dir())?;
    }

    let mut temp_dir = base_dir.join(generate_symlink_path());

    while temp_dir.exists() {
        temp_dir = base_dir.join(generate_symlink_path());
    }

    match symlink_dir(config.default_version_dir(), &temp_dir) {
        Ok(()) => Ok(temp_dir),
        Err(source) => Err(Error::CantCreateSymlink { source, temp_dir }),
    }
}

#[inline]
fn bool_as_str(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

fn env_vars(config: &NvcConfig, multishell_path: &std::path::Path) -> Vec<(&'static str, String)> {
    let base_dir = config.base_dir_with_default();
    let global_prefix_dir = config.global_prefix_dir();

    vec![
        (
            "NVC_MULTISHELL_PATH",
            multishell_path.to_str().unwrap().to_string(),
        ),
        (
            "NVC_VERSION_FILE_STRATEGY",
            config.version_file_strategy().as_str().to_string(),
        ),
        ("NVC_DIR", base_dir.to_str().unwrap().to_string()),
        ("NVC_LOGLEVEL", config.log_level().as_str().to_string()),
        (
            "NVC_NODE_DIST_MIRROR",
            config.node_dist_mirror.as_str().to_string(),
        ),
        (
            "NVC_COREPACK_ENABLED",
            bool_as_str(config.corepack_enabled()).to_string(),
        ),
        (
            "NVC_RESOLVE_ENGINES",
            bool_as_str(config.resolve_engines()).to_string(),
        ),
        ("NVC_ARCH", config.arch.as_str().to_string()),
        (
            "NPM_CONFIG_PREFIX",
            global_prefix_dir.to_str().unwrap().to_string(),
        ),
    ]
}

fn path_entries(config: &NvcConfig, multishell_path: &std::path::Path) -> [std::path::PathBuf; 2] {
    let global_bin = config.global_bin_dir();
    let node_bin = if cfg!(windows) {
        multishell_path.to_path_buf()
    } else {
        multishell_path.join("bin")
    };

    [global_bin, node_bin]
}

impl Command for Env {
    type Error = Error;

    fn apply(self, config: &NvcConfig) -> Result<(), Self::Error> {
        if self.multi {
            outln!(
                config,
                Error,
                "{} {} is deprecated. This is now the default.",
                "warning:".yellow().bold(),
                "--multi".italic()
            );
        }

        let multishell_path = make_symlink(config)?;
        std::fs::create_dir_all(config.global_prefix_dir())?;
        if !cfg!(windows) {
            std::fs::create_dir_all(config.global_bin_dir())?;
        }
        let env_vars = env_vars(config, &multishell_path);

        if self.json {
            let env_var_map: HashMap<_, _> = env_vars.iter().cloned().collect();
            println!("{}", serde_json::to_string(&env_var_map).unwrap());
            return Ok(());
        }

        let shell: Box<dyn Shell> = self
            .shell
            .map(Into::into)
            .or_else(infer_shell)
            .ok_or(Error::CantInferShell)?;

        for path in path_entries(config, &multishell_path) {
            println!("{}", shell.path(&path)?);
        }

        for (name, value) in &env_vars {
            println!("{}", shell.set_env_var(name, value));
        }

        if self.use_on_cd {
            // Call `use` internally for the initial directory, so the shell doesn't
            // need to spawn a subprocess after evaluating the env output.
            let config_with_multishell = config.clone().with_multishell_path(multishell_path);
            let use_cmd = Use {
                version: None,
                install_if_missing: false,
                silent_if_unchanged: true,
                info_to_stderr: true,
                skip_path_check: true,
            };
            let should_force_stderr_color = !std::io::stdout().is_terminal()
                && std::io::stderr().is_terminal()
                && std::env::var_os("NO_COLOR").is_none();
            if should_force_stderr_color {
                colored::control::set_override(true);
            }
            // Ignore errors - if there's no version file, that's fine
            let _ = use_cmd.apply(&config_with_multishell);
            if should_force_stderr_color {
                colored::control::unset_override();
            }

            println!("{}", shell.use_on_cd(config)?);
        }
        if let Some(v) = shell.rehash() {
            println!("{v}");
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(
        "{}\n{}\n{}\n{}",
        "Can't infer shell!",
        "nvc can't infer your shell based on the process tree.",
        "Maybe it is unsupported? we support the following shells:",
        shells_as_string()
    )]
    CantInferShell,
    #[error("Can't create the symlink for multishells at {temp_dir:?}. Maybe there are some issues with permissions for the directory? {source}")]
    CantCreateSymlink {
        #[source]
        source: std::io::Error,
        temp_dir: std::path::PathBuf,
    },
    #[error(transparent)]
    ShellError {
        #[from]
        source: anyhow::Error,
    },
    #[error(transparent)]
    IoError {
        #[from]
        source: std::io::Error,
    },
}

fn shells_as_string() -> String {
    Shells::value_variants()
        .iter()
        .map(|x| format!("* {x}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_vars_include_npm_config_prefix() {
        let base_dir = std::env::temp_dir().join("nvc-config-test");
        let multishell_path = std::env::temp_dir().join("nvc-multishell");
        let config = NvcConfig::default().with_base_dir(Some(base_dir.clone()));

        let vars: HashMap<_, _> = env_vars(&config, &multishell_path).into_iter().collect();

        assert_eq!(
            vars.get("NPM_CONFIG_PREFIX").unwrap(),
            &base_dir.join("global").to_str().unwrap().to_string()
        );
        assert_eq!(
            vars.get("NVC_MULTISHELL_PATH").unwrap(),
            &multishell_path.to_str().unwrap().to_string()
        );
    }

    #[test]
    fn test_path_entries_put_global_before_multishell_bin() {
        let base_dir = std::env::temp_dir().join("nvc-path-test");
        let multishell_path = std::env::temp_dir().join("nvc-multishell");
        let config = NvcConfig::default().with_base_dir(Some(base_dir.clone()));
        let paths = path_entries(&config, &multishell_path);

        if cfg!(windows) {
            assert_eq!(paths[0], base_dir.join("global"));
            assert_eq!(paths[1], multishell_path);
        } else {
            assert_eq!(paths[0], base_dir.join("global").join("bin"));
            assert_eq!(paths[1], multishell_path.join("bin"));
        }
    }

    #[test]
    fn test_smoke() {
        let config = NvcConfig::default();
        Env {
            #[cfg(windows)]
            shell: Some(Shells::Cmd),
            #[cfg(not(windows))]
            shell: Some(Shells::Bash),
            ..Default::default()
        }
        .call(config);
    }
}
