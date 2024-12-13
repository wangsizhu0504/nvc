pub mod extract;

#[cfg(unix)]
pub mod tar_xz;
#[cfg(windows)]
pub mod zip;

pub use self::extract::{Error, Extract};
pub use self::tar_xz::TarXz;