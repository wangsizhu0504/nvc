use super::command::Command;
use crate::config::NvcConfig;
use crate::current_version::{current_version, Error as CurrentVersionError};
use crate::outln;
use colored::Colorize;
use thiserror::Error;

const VERSION_FILE_NAME: &str = ".node-version";

#[derive(clap::Parser, Debug)]
pub struct Pin {
    /// Version string to write. Defaults to the active nvc version.
    version: Option<String>,
}

impl Command for Pin {
    type Error = Error;

    fn apply(self, config: &NvcConfig) -> Result<(), Self::Error> {
        let version = match self.version {
            Some(version) => normalize_explicit_version(&version)?,
            None => current_version(config)?
                .ok_or(Error::NoCurrentVersion)?
                .v_str(),
        };
        let version_file = std::env::current_dir()?.join(VERSION_FILE_NAME);

        std::fs::write(&version_file, format!("{version}\n"))?;
        outln!(
            config,
            Info,
            "Pinned Node version {} to {}",
            version.cyan(),
            VERSION_FILE_NAME.cyan()
        );

        Ok(())
    }
}

fn normalize_explicit_version(version: &str) -> Result<String, Error> {
    let version = version.trim();
    if version.is_empty() {
        return Err(Error::EmptyVersion);
    }
    if version.contains(['\n', '\r']) {
        return Err(Error::MultilineVersion);
    }

    Ok(version.to_string())
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Version cannot be empty")]
    EmptyVersion,
    #[error("Version cannot contain newlines")]
    MultilineVersion,
    #[error("No active nvc version found. Provide a version explicitly, like `nvc pin 22`.")]
    NoCurrentVersion,
    #[error(transparent)]
    CurrentVersion {
        #[from]
        source: CurrentVersionError,
    },
    #[error(transparent)]
    Io {
        #[from]
        source: std::io::Error,
    },
}
