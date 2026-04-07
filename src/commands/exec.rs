use super::command::Command as Cmd;
use crate::choose_version_for_user_input::{
    choose_version_for_user_input, Error as UserInputError,
};
use crate::config::NvcConfig;
use crate::outln;
use crate::user_version::UserVersion;
use crate::user_version_reader::UserVersionReader;
use colored::Colorize;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use thiserror::Error;

#[derive(Debug, clap::Parser)]
#[clap(trailing_var_arg = true)]
pub struct Exec {
    /// Either an explicit version, or a filename with the version written in it
    #[clap(long = "using")]
    version: Option<UserVersionReader>,
    /// Deprecated. This is the default now.
    #[clap(long = "using-file", hide = true)]
    using_file: bool,
    /// The command to run
    arguments: Vec<String>,
}

fn node_bin_path(installation_path: &Path) -> PathBuf {
    if cfg!(windows) {
        installation_path.to_path_buf()
    } else {
        installation_path.join("bin")
    }
}

fn build_path_env(
    current_path: &OsStr,
    node_bin_path: &Path,
    global_bin_path: Option<&Path>,
) -> Result<OsString, std::env::JoinPathsError> {
    let mut paths: Vec<_> = std::env::split_paths(current_path).collect();
    if let Some(global_bin_path) = global_bin_path {
        paths.insert(0, global_bin_path.to_path_buf());
    }
    paths.insert(0, node_bin_path.to_path_buf());
    std::env::join_paths(paths)
}

#[derive(Clone, Copy, Debug)]
enum EnvironmentKind {
    User,
    Internal,
}

impl Exec {
    pub(crate) fn new_for_version(
        version: &crate::version::Version,
        cmd: &str,
        arguments: &[&str],
    ) -> Self {
        let reader = UserVersionReader::Direct(UserVersion::Full(version.clone()));
        let args: Vec<_> = std::iter::once(cmd)
            .chain(arguments.iter().copied())
            .map(String::from)
            .collect();
        Self {
            version: Some(reader),
            using_file: false,
            arguments: args,
        }
    }

    fn run_and_wait_with_environment(
        self,
        config: &NvcConfig,
        environment_kind: EnvironmentKind,
    ) -> Result<ExitStatus, Error> {
        let (binary, arguments) = self
            .arguments
            .split_first()
            .ok_or(Error::NoBinaryProvided)?;

        let version = self
            .version
            .unwrap_or_else(|| {
                let current_dir = std::env::current_dir().unwrap();
                UserVersionReader::Path(current_dir)
            })
            .into_user_version(config)
            .ok_or(Error::CantInferVersion)?;

        let applicable_version = choose_version_for_user_input(&version, config)
            .map_err(|source| Error::ApplicableVersionError { source })?
            .ok_or(Error::VersionNotFound { version })?;

        let paths_env = std::env::var_os("PATH").ok_or(Error::CantReadPathVariable)?;
        let bin_path = node_bin_path(applicable_version.path());
        let global_bin_path = match environment_kind {
            EnvironmentKind::User => {
                std::fs::create_dir_all(config.global_prefix_dir())
                    .map_err(|source| Error::CantPrepareSharedPrefix { source })?;
                if !cfg!(windows) {
                    std::fs::create_dir_all(config.global_bin_dir())
                        .map_err(|source| Error::CantPrepareSharedPrefix { source })?;
                }
                Some(config.global_bin_dir())
            }
            EnvironmentKind::Internal => None,
        };
        let path_env = build_path_env(&paths_env, &bin_path, global_bin_path.as_deref())
            .map_err(|source| Error::CantAddPathToEnvironment { source })?;

        log::debug!(
            "Running {} with PATH={:?} and environment_kind={:?}",
            binary,
            path_env,
            environment_kind
        );

        let mut command = Command::new(binary);
        command
            .args(arguments)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .env("PATH", path_env);
        if let EnvironmentKind::User = environment_kind {
            command.env("NPM_CONFIG_PREFIX", config.global_prefix_dir());
        }

        command
            .spawn()
            .map_err(|source| Error::CantSpawnProgram {
                source,
                binary: binary.to_string(),
            })?
            .wait()
            .map_err(|source| Error::CantWaitForProgram { source })
    }

    pub(crate) fn run_and_wait(self, config: &NvcConfig) -> Result<ExitStatus, Error> {
        self.run_and_wait_with_environment(config, EnvironmentKind::User)
    }

    pub(crate) fn run_and_wait_internal(self, config: &NvcConfig) -> Result<ExitStatus, Error> {
        self.run_and_wait_with_environment(config, EnvironmentKind::Internal)
    }
}

impl Cmd for Exec {
    type Error = Error;

    fn apply(self, config: &NvcConfig) -> Result<(), Self::Error> {
        if self.using_file {
            outln!(
                config,
                Error,
                "{} {} is deprecated. This is now the default.",
                "warning:".yellow().bold(),
                "--using-file".italic()
            );
        }

        let exit_status = self.run_and_wait(config)?;
        let code = exit_status.code().ok_or(Error::CantReadProcessExitCode)?;
        std::process::exit(code);
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Can't spawn program: {source}\nMaybe the program {} does not exist or is not available in PATH?", binary.bold())]
    CantSpawnProgram {
        source: std::io::Error,
        binary: String,
    },
    #[error("Can't wait for child process: {}", source)]
    CantWaitForProgram { source: std::io::Error },
    #[error("Can't prepare shared npm prefix directory: {}", source)]
    CantPrepareSharedPrefix { source: std::io::Error },
    #[error("Can't read path environment variable")]
    CantReadPathVariable,
    #[error("Can't add path to environment variable: {}", source)]
    CantAddPathToEnvironment { source: std::env::JoinPathsError },
    #[error("Can't find version in dotfiles. Please provide a version manually to the command.")]
    CantInferVersion,
    #[error("Requested version {} is not currently installed", version)]
    VersionNotFound { version: UserVersion },
    #[error(transparent)]
    ApplicableVersionError {
        #[from]
        source: UserInputError,
    },
    #[error("Can't read exit code from process.\nMaybe the process was killed using a signal?")]
    CantReadProcessExitCode,
    #[error("command not provided. Please provide a command to run as an argument, like {} or {}.\n{} {}", "node".italic(), "bash".italic(), "example:".yellow().bold(), "nvc exec --using=12 node --version".italic().yellow())]
    NoBinaryProvided,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_path_env_puts_node_before_global_before_existing_path() {
        let old_path_a = std::env::temp_dir().join("old-bin-a");
        let old_path_b = std::env::temp_dir().join("old-bin-b");
        let existing_path = std::env::join_paths([old_path_a.clone(), old_path_b.clone()]).unwrap();
        let node_bin = if cfg!(windows) {
            std::env::temp_dir().join("node")
        } else {
            std::env::temp_dir().join("node").join("bin")
        };
        let global_bin = if cfg!(windows) {
            std::env::temp_dir().join("global")
        } else {
            std::env::temp_dir().join("global").join("bin")
        };

        let path_env = build_path_env(&existing_path, &node_bin, Some(&global_bin)).unwrap();
        let paths: Vec<_> = std::env::split_paths(&path_env).collect();

        assert_eq!(paths[0], node_bin);
        assert_eq!(paths[1], global_bin);
        assert_eq!(paths[2], old_path_a);
        assert_eq!(paths[3], old_path_b);
    }

    #[test]
    fn test_build_path_env_can_skip_global_path_for_internal_commands() {
        let old_path = std::env::temp_dir().join("old-bin");
        let existing_path = std::env::join_paths([old_path.clone()]).unwrap();
        let node_bin = if cfg!(windows) {
            std::env::temp_dir().join("node")
        } else {
            std::env::temp_dir().join("node").join("bin")
        };

        let path_env = build_path_env(&existing_path, &node_bin, None).unwrap();
        let paths: Vec<_> = std::env::split_paths(&path_env).collect();

        assert_eq!(paths[0], node_bin);
        assert_eq!(paths[1], old_path);
    }
}
