# nvc

[简体中文](./README.zh-CN.md)

`nvc` is a cross-platform Node.js version manager focused on predictable shell integration and reproducible installs.

## Overview

- **Status**: Actively maintained
- **Positioning**: Independent project inspired by `fnm` behavior, not a direct code mirror
- **Platforms**: macOS, Linux, Windows
- **Shells**: Bash, Zsh, Fish, PowerShell, Windows Cmd

## Features

- Fast single-binary CLI
- Supports `.node-version`, `.nvmrc`, and `package.json` engine resolution
- Shared global npm prefix across installed Node versions
- Checksum verification before activating downloaded Node archives
- Built-in maintenance commands (`doctor`, `cache`, `prune`)

## Installation

### Install Script (macOS/Linux)

```sh
curl -o- https://raw.githubusercontent.com/wangsizhu0504/nvc/master/install.sh | bash
```

Dependencies:

- Linux: `curl`, `unzip`
- macOS (default path): `curl`, `unzip`, `brew` (script uses Homebrew by default)
- macOS with `--force-install`: `curl`, `unzip`

Common script flags:

- `--install-dir <path>`
- `--skip-shell`
- `--force-install` (alias: `--force-no-brew`)
- `--release <tag|latest>`

### Release Binary

Download the matching binary from [GitHub Releases](https://github.com/wangsizhu0504/nvc/releases), place it on `PATH`, then run shell setup.

### Cargo

```sh
cargo install nvc
```

### Homebrew

```sh
brew install wangsizhu0504/tap/nvc
```

## Shell Setup

Add one of the following to your shell profile.

### Bash / Zsh

```sh
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

## Common Commands

```sh
# list versions
nvc list-remote
nvc list

# install / switch
nvc install <version>
nvc use <version>
nvc current
nvc pin <version>

# run with a specific runtime
nvc exec --using=<version> node --version

# diagnostics and cleanup
nvc doctor
nvc cache dir
nvc cache size --bytes
nvc cache clear
nvc prune --dry-run
nvc prune --all

# update nvc itself
nvc self update
nvc self update --version v1.0.2
```

## Release Process

- Official release tags use `vX.Y.Z`
- Release workflow can be triggered by:
  - pushing a `vX.Y.Z` tag, or
  - manually dispatching the `Release` workflow with an existing tag
- Pipeline runs quality checks, builds macOS/Linux/Windows binaries, generates `checksums.txt`, then publishes GitHub Release assets

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

Related docs:

- [Testing Strategy](./docs/testing.md)
- [Release Checklist](./docs/release-checklist.md)
- [Maintainer Policy](./docs/maintainer-policy.md)
- [Support Policy](./docs/support-policy.md)

## License

[MIT](./LICENSE) © 2024-PRESENT [Kriszu](https://github.com/wangsizhu0504)
