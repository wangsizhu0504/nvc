use crate::version_file_strategy::VersionFileStrategy;

use super::shell::Shell;
use indoc::formatdoc;
use std::path::Path;

#[derive(Debug)]
pub struct Bash;

impl Shell for Bash {
    fn to_clap_shell(&self) -> clap_complete::Shell {
        clap_complete::Shell::Bash
    }

    fn path(&self, path: &Path) -> anyhow::Result<String> {
        let path = path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Can't convert path to string"))?;
        let path =
            super::windows_compat::maybe_fix_windows_path(path).unwrap_or_else(|| path.to_string());
        Ok(format!("export PATH={path:?}:\"$PATH\""))
    }

    fn set_env_var(&self, name: &str, value: &str) -> String {
        format!("export {name}={value:?}")
    }

    fn use_on_cd(&self, config: &crate::config::NvcConfig) -> anyhow::Result<String> {
        let autoload_hook = match config.version_file_strategy() {
            VersionFileStrategy::Local | VersionFileStrategy::Recursive => {
                String::from(r"nvc use --silent-if-unchanged")
            }
        };
        Ok(formatdoc!(
            r#"
                __nvc_use_if_file_found() {{
                    {autoload_hook}
                }}

                __nvccd() {{
                    \cd "$@" || return $?
                    __nvc_use_if_file_found
                }}

                alias cd=__nvccd
                __nvc_use_if_file_found
            "#,
            autoload_hook = autoload_hook
        ))
    }
}
