use crate::arch::Arch;
use crate::log_level::LogLevel;
use crate::path_ext::PathExt;
use crate::version_file_strategy::VersionFileStrategy;
use dirs::{data_dir, home_dir};
use url::Url;

#[derive(clap::Parser, Debug)]
pub struct NvcConfig {
    /// <https://nodejs.org/dist/> mirror
    #[clap(
        long,
        env = "NVC_NODE_DIST_MIRROR",
        default_value = "https://nodejs.org/dist",
        global = true,
        hide_env_values = true
    )]
    pub node_dist_mirror: Url,

    /// The root directory of nvc installations.
    #[clap(
        long = "nvc-dir",
        env = "NVC_DIR",
        global = true,
        hide_env_values = true
    )]
    pub base_dir: Option<std::path::PathBuf>,

    /// Where the current node version link is stored.
    /// This value will be populated automatically by evaluating
    /// `nvc env` in your shell profile. Read more about it using `nvc help env`
    #[clap(long, env = "NVC_MULTISHELL_PATH", hide_env_values = true, hide = true)]
    multishell_path: Option<std::path::PathBuf>,

    /// The log level of nvc commands
    #[clap(
        long,
        env = "NVC_LOGLEVEL",
        default_value_t,
        global = true,
        hide_env_values = true
    )]
    log_level: LogLevel,

    /// Override the architecture of the installed Node binary.
    /// Defaults to arch of nvc binary.
    #[clap(
        long,
        env = "NVC_ARCH",
        default_value_t,
        global = true,
        hide_env_values = true,
        hide_default_value = true
    )]
    pub arch: Arch,

    /// A strategy for how to resolve the Node version. Used whenever `nvc use` or `nvc install` is
    /// called without a version, or when `--use-on-cd` is configured on evaluation.
    #[clap(
        long,
        env = "NVC_VERSION_FILE_STRATEGY",
        default_value_t,
        global = true,
        hide_env_values = true
    )]
    version_file_strategy: VersionFileStrategy,

    /// Enable corepack support for each new installation.
    /// This will make nvc call `corepack enable` on every Node.js installation.
    /// For more information about corepack see <https://nodejs.org/api/corepack.html>
    #[clap(
        long,
        env = "NVC_COREPACK_ENABLED",
        global = true,
        hide_env_values = true
    )]
    corepack_enabled: bool,

    /// Resolve `engines.node` field in `package.json` whenever a `.node-version` or `.nvmrc` file is not present.
    /// Experimental: This feature is subject to change.
    /// Note: `engines.node` can be any semver range, with the latest satisfying version being resolved.
    #[clap(
        long,
        env = "NVC_RESOLVE_ENGINES",
        global = true,
        hide_env_values = true,
        verbatim_doc_comment
    )]
    resolve_engines: bool,
}

impl Default for NvcConfig {
    fn default() -> Self {
        Self {
            node_dist_mirror: Url::parse("https://nodejs.org/dist/").unwrap(),
            base_dir: None,
            multishell_path: None,
            log_level: LogLevel::Info,
            arch: Arch::default(),
            version_file_strategy: VersionFileStrategy::default(),
            corepack_enabled: false,
            resolve_engines: false,
        }
    }
}

impl NvcConfig {
    pub fn version_file_strategy(&self) -> &VersionFileStrategy {
        &self.version_file_strategy
    }

    pub fn corepack_enabled(&self) -> bool {
        self.corepack_enabled
    }

    pub fn resolve_engines(&self) -> bool {
        self.resolve_engines
    }

    pub fn multishell_path(&self) -> Option<&std::path::Path> {
        match &self.multishell_path {
            None => None,
            Some(v) => Some(v.as_path()),
        }
    }

    pub fn log_level(&self) -> &LogLevel {
        &self.log_level
    }

    pub fn base_dir_with_default(&self) -> std::path::PathBuf {
        let user_pref = self.base_dir.clone();
        if let Some(dir) = user_pref {
            return dir;
        }

        let legacy = home_dir()
            .map(|dir| dir.join(".nvc"))
            .filter(|dir| dir.exists());

        let modern = data_dir().map(|dir| dir.join("nvc"));

        if let Some(dir) = legacy {
            return dir;
        }

        modern
            .expect("Can't get data directory")
            .ensure_exists_silently()
    }

    pub fn installations_dir(&self) -> std::path::PathBuf {
        self.base_dir_with_default()
            .join("node-versions")
            .ensure_exists_silently()
    }

    pub fn default_version_dir(&self) -> std::path::PathBuf {
        self.aliases_dir().join("default")
    }

    pub fn aliases_dir(&self) -> std::path::PathBuf {
        self.base_dir_with_default()
            .join("aliases")
            .ensure_exists_silently()
    }

    #[cfg(test)]
    pub fn with_base_dir(mut self, base_dir: Option<std::path::PathBuf>) -> Self {
        self.base_dir = base_dir;
        self
    }
}
