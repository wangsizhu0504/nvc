use super::command::Command;
use crate::cache_ops;
use crate::config::NvcConfig;
use clap::ValueEnum;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
#[clap(name = "cache", bin_name = "cache")]
pub struct Cache {
    /// Cache action to perform
    #[arg(value_enum)]
    action: CacheAction,

    /// Print the size as raw bytes
    #[clap(long, requires = "action")]
    bytes: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum CacheAction {
    /// Print the download cache directory
    Dir,
    /// Print the current download cache size
    Size,
    /// Clear the download cache directory
    Clear,
}

impl Command for Cache {
    type Error = Error;

    fn apply(self, config: &NvcConfig) -> Result<(), Self::Error> {
        match self.action {
            CacheAction::Dir => {
                println!("{}", cache_ops::downloads_dir(config).display());
            }
            CacheAction::Size => {
                let size = cache_ops::dir_size(&cache_ops::downloads_dir(config))?;
                if self.bytes {
                    println!("{size}");
                } else {
                    println!("{size} bytes");
                }
            }
            CacheAction::Clear => {
                let result = cache_ops::clear_downloads(config, false)?;
                println!(
                    "Removed {} cache entr{}",
                    result.removed_count(),
                    if result.removed_count() == 1 {
                        "y"
                    } else {
                        "ies"
                    }
                );
            }
        }

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
