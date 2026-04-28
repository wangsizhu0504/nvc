use super::command::Command;
use crate::config::NvcConfig;
use crate::outln;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::io::Seek;
use std::path::Path;
use tempfile::NamedTempFile;
use thiserror::Error;

const LATEST_RELEASE_API_URL: &str =
    "https://api.github.com/repos/wangsizhu0504/nvc/releases/latest";
const RELEASE_DOWNLOAD_BASE_URL: &str = "https://github.com/wangsizhu0504/nvc/releases/download";

#[derive(clap::Parser, Debug)]
#[clap(name = "self", bin_name = "nvc self")]
pub struct SelfCommand {
    #[clap(subcommand)]
    action: SelfAction,
}

#[derive(clap::Subcommand, Debug)]
enum SelfAction {
    /// Update the nvc executable
    #[clap(name = "update", bin_name = "nvc self update")]
    Update(Update),
}

#[derive(clap::Parser, Debug)]
#[clap(name = "update", bin_name = "nvc self update")]
struct Update {
    /// Release tag to install. Defaults to the latest GitHub release.
    #[clap(long)]
    version: Option<String>,
}

impl Command for SelfCommand {
    type Error = Error;

    fn apply(self, config: &NvcConfig) -> Result<(), Self::Error> {
        match self.action {
            SelfAction::Update(update) => update.apply(config),
        }
    }
}

impl Update {
    fn apply(self, config: &NvcConfig) -> Result<(), Error> {
        let tag = match self.version {
            Some(version) => normalize_release_tag(&version)?,
            None => latest_release_tag()?,
        };
        let filename = release_asset_filename(&tag)?;
        let url = format!("{RELEASE_DOWNLOAD_BASE_URL}/{tag}/{filename}");
        let expected_checksum = fetch_expected_checksum(&tag, &filename)?;
        let current_exe = std::env::current_exe()?;

        replace_current_exe(&url, &current_exe, &expected_checksum)?;
        outln!(config, Info, "Updated nvc to {tag}");

        Ok(())
    }
}

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

fn latest_release_tag() -> Result<String, Error> {
    let response = crate::http::get(LATEST_RELEASE_API_URL)?
        .error_for_status()
        .map_err(crate::http::Error::from)?;
    let text = response.text().map_err(crate::http::Error::from)?;
    let release: GitHubRelease = serde_json::from_str(&text)?;

    normalize_release_tag(&release.tag_name)
}

fn normalize_release_tag(tag: &str) -> Result<String, Error> {
    let tag = tag.trim();
    if tag.is_empty() {
        return Err(Error::EmptyVersion);
    }
    if tag.contains(['\n', '\r', '/']) {
        return Err(Error::InvalidVersion {
            version: tag.into(),
        });
    }
    if tag.starts_with('v') {
        Ok(tag.to_string())
    } else {
        Ok(format!("v{tag}"))
    }
}

fn release_asset_filename(tag: &str) -> Result<String, Error> {
    let target = release_target().ok_or(Error::UnsupportedTarget)?;
    let extension = if cfg!(windows) { ".exe" } else { "" };
    Ok(format!("nvc-{tag}-{target}{extension}"))
}

fn fetch_expected_checksum(tag: &str, filename: &str) -> Result<String, Error> {
    let url = format!("{RELEASE_DOWNLOAD_BASE_URL}/{tag}/checksums.txt");
    let response = crate::http::get(&url)?
        .error_for_status()
        .map_err(crate::http::Error::from)?;
    let text = response.text().map_err(crate::http::Error::from)?;
    parse_checksum(&text, filename).ok_or_else(|| Error::MissingChecksum {
        filename: filename.into(),
    })
}

fn parse_checksum(manifest: &str, filename: &str) -> Option<String> {
    manifest.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        match (parts.next(), parts.next()) {
            (Some(checksum), Some(entry_filename)) if entry_filename == filename => {
                Some(checksum.to_string())
            }
            _ => None,
        }
    })
}

fn release_target() -> Option<&'static str> {
    if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Some("x86_64-unknown-linux-musl")
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        Some("aarch64-apple-darwin")
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        Some("x86_64-apple-darwin")
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        Some("x86_64-pc-windows-msvc")
    } else {
        None
    }
}

fn replace_current_exe(
    url: &str,
    current_exe: &Path,
    expected_checksum: &str,
) -> Result<(), Error> {
    let parent = current_exe
        .parent()
        .ok_or_else(|| Error::InvalidCurrentExePath {
            path: current_exe.into(),
        })?;
    let mut response = crate::http::get(url)?
        .error_for_status()
        .map_err(crate::http::Error::from)?;
    let mut temp_file = NamedTempFile::new_in(parent)?;
    std::io::copy(&mut response, temp_file.as_file_mut())?;
    verify_checksum(&mut temp_file, expected_checksum)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mode = std::fs::metadata(current_exe)?.permissions().mode();
        temp_file
            .as_file_mut()
            .set_permissions(std::fs::Permissions::from_mode(mode))?;
    }

    temp_file.as_file_mut().rewind()?;
    temp_file
        .persist(current_exe)
        .map_err(|source| Error::CantReplaceCurrentExe {
            source: source.error,
        })?;
    Ok(())
}

fn verify_checksum(temp_file: &mut NamedTempFile, expected_checksum: &str) -> Result<(), Error> {
    let mut hasher = Sha256::new();
    temp_file.as_file_mut().rewind()?;
    std::io::copy(temp_file.as_file_mut(), &mut hasher)?;
    temp_file.as_file_mut().rewind()?;

    let actual_checksum = format!("{:x}", hasher.finalize());
    if actual_checksum != expected_checksum {
        return Err(Error::ChecksumMismatch {
            expected: expected_checksum.into(),
            actual: actual_checksum,
        });
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Version cannot be empty")]
    EmptyVersion,
    #[error("Invalid release tag: {version}")]
    InvalidVersion { version: String },
    #[error("No release asset is available for this platform")]
    UnsupportedTarget,
    #[error("Can't read current nvc executable path: {path}")]
    InvalidCurrentExePath { path: std::path::PathBuf },
    #[error("Can't replace current nvc executable: {source}")]
    CantReplaceCurrentExe { source: std::io::Error },
    #[error("checksums.txt does not contain an entry for {filename}")]
    MissingChecksum { filename: String },
    #[error("Downloaded binary checksum mismatch. Expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },
    #[error(transparent)]
    Http {
        #[from]
        source: crate::http::Error,
    },
    #[error(transparent)]
    Json {
        #[from]
        source: serde_json::Error,
    },
    #[error(transparent)]
    Io {
        #[from]
        source: std::io::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_version_is_normalized_to_release_tag() {
        assert_eq!(normalize_release_tag("1.0.2").unwrap(), "v1.0.2");
        assert_eq!(normalize_release_tag("v1.0.2").unwrap(), "v1.0.2");
    }

    #[test]
    fn release_asset_filename_contains_tag_and_supported_target() {
        let filename = release_asset_filename("v1.0.2").unwrap();

        assert!(filename.starts_with("nvc-v1.0.2-"));
    }

    #[test]
    fn parse_checksum_finds_matching_asset() {
        let manifest = "\
abc123  nvc-v1.0.2-x86_64-unknown-linux-musl
def456  nvc-v1.0.2-aarch64-apple-darwin
";

        assert_eq!(
            parse_checksum(manifest, "nvc-v1.0.2-aarch64-apple-darwin"),
            Some("def456".into())
        );
    }
}
