use thiserror::Error;

use crate::config::NvcConfig;
use crate::system_version;
use crate::version::Version;

pub fn current_version(config: &NvcConfig) -> Result<Option<Version>, Error> {
    let multishell_path = config.multishell_path().ok_or(Error::EnvNotApplied)?;

    if multishell_path.read_link().ok() == Some(system_version::path()) {
        return Ok(Some(Version::Bypassed));
    }

    let Ok(resolved_path) = std::fs::canonicalize(multishell_path) else {
        return Ok(None);
    };

    let installation_path = resolved_path
        .parent()
        .ok_or_else(|| Error::InvalidMultishellPath {
            path: resolved_path.clone(),
        })?;
    let file_name = installation_path
        .file_name()
        .ok_or_else(|| Error::InvalidMultishellPath {
            path: resolved_path.clone(),
        })?
        .to_str()
        .ok_or_else(|| Error::InvalidMultishellPath {
            path: resolved_path.clone(),
        })?;
    let version = Version::parse(file_name).map_err(|source| Error::VersionError {
        source,
        version: file_name.to_string(),
    })?;

    Ok(Some(version))
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("`nvc env` was not applied in this context.\nCan't find nvc's environment variables")]
    EnvNotApplied,
    #[error("Can't read the active Node version from multishell path {}", path.display())]
    InvalidMultishellPath { path: std::path::PathBuf },
    #[error("Can't read the version as a valid semver from {version}")]
    VersionError {
        source: node_semver::SemverError,
        version: String,
    },
}
