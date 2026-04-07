use crate::arch::Arch;
use crate::archive::{Archive, Error as ExtractError};
use crate::directory_portal::DirectoryPortal;
use crate::progress::ResponseProgress;
use crate::version::Version;
use indicatif::ProgressDrawTarget;
use log::debug;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use std::path::PathBuf;
use tempfile::NamedTempFile;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    HttpError {
        #[from]
        source: crate::http::Error,
    },
    #[error(transparent)]
    IoError {
        #[from]
        source: std::io::Error,
    },
    #[error("Can't extract the file: {}", source)]
    CantExtractFile {
        #[from]
        source: ExtractError,
    },
    #[error("The downloaded archive is empty")]
    TarIsEmpty,
    #[error("{} for {} not found upstream.\nYou can `nvc ls-remote` to see available versions or try a different `--arch`.", version, arch)]
    VersionNotFound { version: Version, arch: Arch },
    #[error("Version already installed at {:?}", path)]
    VersionAlreadyInstalled { path: PathBuf },
    #[error("Can't fetch checksum manifest for {}: {}", version, source)]
    ChecksumManifestError {
        version: Version,
        source: crate::http::Error,
    },
    #[error(
        "Checksum manifest for {} does not contain an entry for {}",
        version,
        filename
    )]
    MissingChecksumEntry { version: Version, filename: String },
    #[error(
        "Checksum mismatch for {}. Expected {}, got {}",
        filename,
        expected,
        actual
    )]
    ChecksumMismatch {
        filename: String,
        expected: String,
        actual: String,
    },
}

#[cfg(unix)]
fn filename_for_version(version: &Version, arch: Arch, ext: &str) -> String {
    format!(
        "node-{node_ver}-{platform}-{arch}.{ext}",
        node_ver = &version,
        platform = crate::system_info::platform_name(),
        arch = arch,
        ext = ext
    )
}

#[cfg(windows)]
fn filename_for_version(version: &Version, arch: Arch, ext: &str) -> String {
    format!(
        "node-{node_ver}-win-{arch}.{ext}",
        node_ver = &version,
        arch = arch,
        ext = ext,
    )
}

fn download_url(base_url: &Url, version: &Version, filename: &str) -> Url {
    Url::parse(&format!(
        "{}/{}/{}",
        base_url.as_str().trim_end_matches('/'),
        version,
        filename
    ))
    .unwrap()
}

fn checksum_manifest_url(base_url: &Url, version: &Version) -> Url {
    download_url(base_url, version, "SHASUMS256.txt")
}

fn fetch_checksums(
    node_dist_mirror: &Url,
    version: &Version,
) -> Result<HashMap<String, String>, Error> {
    let response = crate::http::get(checksum_manifest_url(node_dist_mirror, version).as_str())
        .and_then(|response| {
            response
                .error_for_status()
                .map_err(crate::http::Error::from)
        })
        .map_err(|source| Error::ChecksumManifestError {
            version: version.clone(),
            source,
        })?;

    parse_checksums(BufReader::new(response))
}

fn parse_checksums<R: Read>(reader: BufReader<R>) -> Result<HashMap<String, String>, Error> {
    let mut checksums = HashMap::new();
    for line in reader.lines() {
        let line = line?;
        let mut parts = line.split_whitespace();
        if let (Some(checksum), Some(filename)) = (parts.next(), parts.next()) {
            checksums.insert(filename.to_string(), checksum.to_string());
        }
    }

    Ok(checksums)
}

fn download_to_temp_file(
    response: crate::http::Response,
    show_progress: bool,
    temp_installations_dir: &Path,
) -> Result<NamedTempFile, Error> {
    let mut temp_file = NamedTempFile::new_in(temp_installations_dir)?;

    if show_progress {
        let mut progress = ResponseProgress::new(response, ProgressDrawTarget::stderr());
        std::io::copy(&mut progress, temp_file.as_file_mut())?;
    } else {
        let mut response = response;
        std::io::copy(&mut response, temp_file.as_file_mut())?;
    }

    temp_file.as_file_mut().seek(SeekFrom::Start(0))?;
    Ok(temp_file)
}

fn verify_checksum(
    temp_file: &mut NamedTempFile,
    filename: &str,
    expected_checksum: &str,
) -> Result<(), Error> {
    let mut hasher = Sha256::new();
    temp_file.as_file_mut().seek(SeekFrom::Start(0))?;

    let mut buffer = [0_u8; 8 * 1024];
    loop {
        let read = temp_file.as_file_mut().read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    let actual_checksum = format!("{:x}", hasher.finalize());
    if actual_checksum != expected_checksum {
        return Err(Error::ChecksumMismatch {
            filename: filename.to_string(),
            expected: expected_checksum.to_string(),
            actual: actual_checksum,
        });
    }

    temp_file.as_file_mut().seek(SeekFrom::Start(0))?;
    Ok(())
}

/// Install a Node package
pub fn install_node_dist<P: AsRef<Path>>(
    version: &Version,
    node_dist_mirror: &Url,
    installations_dir: P,
    arch: Arch,
    show_progress: bool,
) -> Result<(), Error> {
    let installation_dir = PathBuf::from(installations_dir.as_ref()).join(version.v_str());

    if installation_dir.exists() {
        return Err(Error::VersionAlreadyInstalled {
            path: installation_dir,
        });
    }

    std::fs::create_dir_all(installations_dir.as_ref())?;

    let temp_installations_dir = installations_dir.as_ref().join(".downloads");
    std::fs::create_dir_all(&temp_installations_dir)?;

    let portal = DirectoryPortal::new_in(&temp_installations_dir, installation_dir);
    let checksums = fetch_checksums(node_dist_mirror, version)?;

    for extract in Archive::supported() {
        let filename = filename_for_version(version, arch, extract.file_extension());
        let url = download_url(node_dist_mirror, version, &filename);
        debug!("Going to call for {}", &url);
        let response = crate::http::get(url.as_str())?;

        if !response.status().is_success() {
            continue;
        }

        let expected_checksum =
            checksums
                .get(&filename)
                .ok_or_else(|| Error::MissingChecksumEntry {
                    version: version.clone(),
                    filename: filename.clone(),
                })?;

        debug!("Downloading {} for checksum verification", filename);
        let mut temp_file =
            download_to_temp_file(response, show_progress, &temp_installations_dir)?;
        verify_checksum(&mut temp_file, &filename, expected_checksum)?;

        debug!("Extracting verified archive {}", filename);
        extract.extract_archive_into(portal.as_ref(), temp_file.reopen()?)?;
        debug!("Extraction completed");

        let installed_directory = std::fs::read_dir(&portal)?
            .next()
            .ok_or(Error::TarIsEmpty)??;
        let installed_directory = installed_directory.path();

        let renamed_installation_dir = portal.join("installation");
        std::fs::rename(installed_directory, renamed_installation_dir)?;

        portal.teleport()?;

        return Ok(());
    }

    Err(Error::VersionNotFound {
        version: version.clone(),
        arch,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::downloader::install_node_dist;
    use crate::version::Version;
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;

    #[test]
    fn test_parse_checksums_contains_expected_file() {
        let manifest = r"
84e9f3f274a89ff82f9f5890bc009a2d697bfca0312fb3dbc248212844bb7e20  node-v12.0.0-aix-ppc64.tar.gz
e9669f62977504c9f8b683c044cc13cb31da01a0efd16d5ca7cd264ed6ad5ae5  node-v12.0.0-darwin-x64.tar.xz
";
        let checksums = parse_checksums(BufReader::new(manifest.as_bytes())).unwrap();

        assert_eq!(
            checksums.get("node-v12.0.0-darwin-x64.tar.xz"),
            Some(&"e9669f62977504c9f8b683c044cc13cb31da01a0efd16d5ca7cd264ed6ad5ae5".to_string())
        );
    }

    #[test_log::test]
    #[ignore = "real-download"]
    fn test_installing_node_12() {
        let installations_dir = tempdir().unwrap();
        let node_path = install_in(installations_dir.path()).join("node");

        let stdout = duct::cmd(node_path.to_str().unwrap(), vec!["--version"])
            .stdout_capture()
            .run()
            .expect("Can't run Node binary")
            .stdout;

        let result = String::from_utf8(stdout).expect("Can't read `node --version` output");

        assert_eq!(result.trim(), "v12.0.0");
    }

    #[test_log::test]
    #[ignore = "real-download"]
    fn test_installing_npm() {
        let installations_dir = tempdir().unwrap();
        let bin_dir = install_in(installations_dir.path());
        let npm_path = bin_dir.join(if cfg!(windows) { "npm.cmd" } else { "npm" });

        let stdout = duct::cmd(npm_path.to_str().unwrap(), vec!["--version"])
            .env("PATH", bin_dir)
            .stdout_capture()
            .run()
            .expect("Can't run npm")
            .stdout;

        let result = String::from_utf8(stdout).expect("Can't read npm output");

        assert_eq!(result.trim(), "6.9.0");
    }

    fn install_in(path: &Path) -> PathBuf {
        let version = Version::parse("12.0.0").unwrap();
        let arch = Arch::X64;
        let node_dist_mirror = Url::parse("https://nodejs.org/dist/").unwrap();
        install_node_dist(&version, &node_dist_mirror, path, arch, false)
            .expect("Can't install Node 12");

        let mut location_path = path.join(version.v_str()).join("installation");

        if cfg!(unix) {
            location_path.push("bin");
        }

        location_path
    }
}
