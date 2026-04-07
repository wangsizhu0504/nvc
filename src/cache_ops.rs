use crate::config::NvcConfig;
use crate::fs;
use crate::system_version;
use std::path::{Path, PathBuf};

pub fn downloads_dir(config: &NvcConfig) -> PathBuf {
    config.installations_dir().join(".downloads")
}

pub fn dir_size(path: &Path) -> std::io::Result<u64> {
    if !path.exists() {
        return Ok(0);
    }

    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.is_file() {
        return Ok(metadata.len());
    }

    let mut total = 0;
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        total += dir_size(&entry.path())?;
    }
    Ok(total)
}

#[derive(Debug, Default, Clone)]
pub struct PruneResult {
    pub removed: Vec<PathBuf>,
    pub skipped: Vec<PathBuf>,
}

impl PruneResult {
    pub fn removed_count(&self) -> usize {
        self.removed.len()
    }
}

pub fn clear_downloads(config: &NvcConfig, dry_run: bool) -> std::io::Result<PruneResult> {
    let dir = downloads_dir(config);
    let mut result = PruneResult::default();
    if !dir.exists() {
        return Ok(result);
    }

    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if dry_run {
            result.removed.push(path);
        } else {
            remove_regular_path(&path)?;
            result.removed.push(path);
        }
    }

    Ok(result)
}

pub fn prune_broken_aliases(config: &NvcConfig, dry_run: bool) -> std::io::Result<PruneResult> {
    let aliases_dir = config.aliases_dir();
    let mut result = PruneResult::default();
    if !aliases_dir.exists() {
        return Ok(result);
    }

    for entry in std::fs::read_dir(&aliases_dir)? {
        let entry = entry?;
        let path = entry.path();
        if alias_is_broken(&path) {
            if dry_run {
                result.removed.push(path);
            } else {
                fs::remove_symlink_dir(&path)?;
                result.removed.push(path);
            }
        } else {
            result.skipped.push(path);
        }
    }

    Ok(result)
}

pub fn prune_stale_multishells(config: &NvcConfig, dry_run: bool) -> std::io::Result<PruneResult> {
    let storage_dir = config.multishell_storage();
    let current_multishell = config.multishell_path().map(PathBuf::from);
    let mut result = PruneResult::default();
    if !storage_dir.exists() {
        return Ok(result);
    }

    for entry in std::fs::read_dir(&storage_dir)? {
        let entry = entry?;
        let path = entry.path();

        if current_multishell
            .as_ref()
            .is_some_and(|current| current == &path)
        {
            result.skipped.push(path);
            continue;
        }

        if is_broken_entry(&path) {
            if dry_run {
                result.removed.push(path);
            } else {
                fs::remove_symlink_dir(&path)?;
                result.removed.push(path);
            }
        } else {
            result.skipped.push(path);
        }
    }

    Ok(result)
}

fn alias_is_broken(path: &Path) -> bool {
    match fs::shallow_read_symlink(path) {
        Ok(target) if target == system_version::path() => false,
        Ok(_) | Err(_) => std::fs::canonicalize(path).is_err(),
    }
}

fn is_broken_entry(path: &Path) -> bool {
    std::fs::canonicalize(path).is_err()
}

fn remove_regular_path(path: &Path) -> std::io::Result<()> {
    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || metadata.is_file() {
        std::fs::remove_file(path)?;
    } else if metadata.is_dir() {
        std::fs::remove_dir_all(path)?;
    }
    Ok(())
}
