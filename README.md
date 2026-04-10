# nvc

`nvc` is a cross-platform Node.js version manager focused on predictable shell integration, reproducible installs, and a self-contained Rust CLI.

## Project Status

- Status: actively maintained
- Scope: Node version installation, selection, shell integration, and shared global npm tooling
- Distribution: release binaries, Cargo install, and shell setup script
- Maintenance model: independent product with upstream-inspired behavior, not a direct code mirror of `fnm`

## Why nvc

- Cross-platform support for macOS, Linux, and Windows
- Single-binary CLI with fast startup
- Works with `.node-version`, `.nvmrc`, and `package.json` engine resolution
- Shared global npm prefix for CLI tools installed with `npm install -g`
- Node downloads are verified against official checksums before activation
- Shell integration for Bash, Zsh, Fish, PowerShell, and Windows Cmd

## Installation

### Recommended

The recommended path is a release binary or the install script for macOS/Linux.

### Install Script (macOS/Linux)

Requirements:

- `curl`
- `unzip`

```sh
curl -o- https://raw.githubusercontent.com/wangsizhu0504/nvc/master/install.sh | bash
```

Optional flags:

- `--install-dir`: install into a custom directory
- `--skip-shell`: do not modify shell startup files
- `--force-install`: force script install on macOS even if Homebrew is preferred

Example:

```sh
curl -o- https://raw.githubusercontent.com/wangsizhu0504/nvc/master/install.sh | bash -s -- --install-dir "$HOME/.nvc" --skip-shell
```

### Release Binary

- Download the matching binary from [GitHub Releases](https://github.com/wangsizhu0504/nvc/releases)
- Put it on `PATH`
- Run shell setup
- Official release tags use the `vX.Y.Z` format
- Pushing a release tag runs validation first, then builds release binaries, generates `checksums.txt`, and publishes the GitHub Release automatically

### Cargo

```sh
cargo install nvc
```

### Homebrew

```sh
brew install nvc
```

## Shell Setup

`nvc` works by exporting environment variables and adjusting `PATH` with the output of `nvc env`.

To enable automatic switching on directory change, use `--use-on-cd`.

### Bash

```bash
eval "$(nvc env --use-on-cd)"
```

### Zsh

```zsh
eval "$(nvc env --use-on-cd)"
```

### Fish

```fish
nvc env --use-on-cd | source
```

### PowerShell

```powershell
nvc env --use-on-cd | Out-String | Invoke-Expression
```

### Windows Cmd

```batch
FOR /f "tokens=*" %i IN ('nvc env --use-on-cd') DO CALL %i
```

## Shared Global Packages

`nvc` exports a shared `NPM_CONFIG_PREFIX`, so packages installed with `npm install -g` are available across installed Node versions.

Behavior:

- Shared prefix directory: `<NVC_DIR>/global`
- If `NVC_DIR` is not set: `<default nvc base dir>/global`
- Global binaries remain available after `nvc use` and `nvc exec`

Operational helpers:

- `nvc doctor` inspects shell setup, active version state, `PATH`, and shared global prefix health
- `nvc cache dir|size|clear` manages download cache state
- `nvc prune` removes stale download artifacts, broken aliases, and stale multishell links

Good fits:

- `typescript`
- `eslint`
- `pnpm`
- `yarn`
- other CLI-focused npm tools

Known limit:

- Packages with native addons or strong Node-version coupling may need reinstall for a specific runtime

## Compatibility

Default support target:

- macOS
- Linux
- Windows

Shell support target:

- Bash
- Zsh
- Fish
- PowerShell
- Windows Cmd

## Troubleshooting

Common checks:

- Run `nvc doctor` for a structured health check of shell setup, PATH, and shared global prefix state
- Run `nvc env` in your shell startup file
- Confirm `nvc env --json` returns the expected `NVC_DIR` and `NVC_MULTISHELL_PATH`
- Verify the active shell session includes the `nvc` bin path on `PATH`
- Re-run `nvc install <version>` if a download was interrupted
- If `nvc install --use` fails, source `nvc env` first and retry from an initialized shell session
- If a mirror serves incomplete or modified artifacts, `nvc install` will stop on checksum verification instead of activating the archive

## Cache and Cleanup

`nvc` now includes built-in maintenance commands for cache visibility and cleanup.

- `nvc cache dir`: print the downloads cache directory
- `nvc cache size --bytes`: show the current downloads cache size
- `nvc cache clear`: clear the downloads cache
- `nvc prune --dry-run`: preview stale state cleanup
- `nvc prune --all`: remove stale downloads, broken aliases, and stale multishell links

## Development

```sh
git clone https://github.com/wangsizhu0504/nvc.git
cd nvc
cargo build
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --bin nvc remote_node_index::tests::test_list -- --ignored --exact --nocapture
cargo test --test shared_global_prefix exec_uses_shared_prefix_and_global_packages_are_shared -- --ignored --exact --nocapture
```

Test tiers and CI expectations are documented in [Testing Strategy](./docs/testing.md).

## Release and Support Policy

- PR checks cover formatting, linting, fast tests, and real-download smoke validation
- Heavier multi-platform real-download regressions run on schedule or before release
- Releases should publish artifacts and checksums together
- Pushing a `vX.Y.Z` tag triggers automated validation, multi-platform builds, checksum generation, and GitHub Release publication after the build jobs succeed
- Breaking behavior changes follow semver and must be documented in the changelog

## Upstream and Licensing Policy

`nvc` is an independent project that references upstream `fnm` behavior selectively.

Rules:

- Upstream behavior may be used as implementation reference
- Upstream code is not merged blindly
- Licensing and fork boundaries are documented explicitly
- Any future upstream alignment must be reviewed against project policy before adoption

See [Maintainer Policy](./docs/maintainer-policy.md), [Support Policy](./docs/support-policy.md), and [Release Checklist](./docs/release-checklist.md).

## Acknowledgements

The project originated as a fork-informed effort inspired by `fnm`, but is maintained as its own product.

## License

[MIT](./LICENSE) License © 2024-PRESENT [Kriszu](https://github.com/wangsizhu0504)
