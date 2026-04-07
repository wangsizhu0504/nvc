use super::command::Command;
use crate::cache_ops;
use crate::config::NvcConfig;
use thiserror::Error;

#[derive(clap::Parser, Debug, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct Prune {
    /// Prune download cache entries
    #[clap(long)]
    downloads: bool,
    /// Remove broken aliases
    #[clap(long)]
    aliases: bool,
    /// Remove stale multishell links
    #[clap(long)]
    multishells: bool,
    /// Prune all supported stale state
    #[clap(long)]
    all: bool,
    /// Show what would be removed without deleting it
    #[clap(long)]
    dry_run: bool,
}

impl Prune {
    fn targets(&self) -> (bool, bool, bool) {
        if self.all || !(self.downloads || self.aliases || self.multishells) {
            (true, true, true)
        } else {
            (self.downloads, self.aliases, self.multishells)
        }
    }
}

impl Command for Prune {
    type Error = Error;

    fn apply(self, config: &NvcConfig) -> Result<(), Self::Error> {
        let (downloads, aliases, multishells) = self.targets();
        let mut total_removed = 0;

        if downloads {
            let result = cache_ops::clear_downloads(config, self.dry_run)?;
            total_removed += result.removed_count();
            println!(
                "{} {} download cache entr{}",
                if self.dry_run {
                    "Would remove"
                } else {
                    "Removed"
                },
                result.removed_count(),
                if result.removed_count() == 1 {
                    "y"
                } else {
                    "ies"
                }
            );
        }

        if aliases {
            let result = cache_ops::prune_broken_aliases(config, self.dry_run)?;
            total_removed += result.removed_count();
            println!(
                "{} {} broken alias{}",
                if self.dry_run {
                    "Would remove"
                } else {
                    "Removed"
                },
                result.removed_count(),
                if result.removed_count() == 1 {
                    ""
                } else {
                    "es"
                }
            );
        }

        if multishells {
            let result = cache_ops::prune_stale_multishells(config, self.dry_run)?;
            total_removed += result.removed_count();
            println!(
                "{} {} stale multishell entr{}",
                if self.dry_run {
                    "Would remove"
                } else {
                    "Removed"
                },
                result.removed_count(),
                if result.removed_count() == 1 {
                    "y"
                } else {
                    "ies"
                }
            );
        }

        println!(
            "{} {} entr{} in total",
            if self.dry_run {
                "Would remove"
            } else {
                "Removed"
            },
            total_removed,
            if total_removed == 1 { "y" } else { "ies" }
        );

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io {
        #[from]
        source: std::io::Error,
    },
}
