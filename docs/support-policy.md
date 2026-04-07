# Support Policy

## Supported Platforms

Primary support target:

- macOS
- Linux
- Windows

## Supported Shells

Primary shell support target:

- Bash
- Zsh
- Fish
- PowerShell
- Windows Cmd

## Quality Model

The project aims to keep:

- formatting and lint checks green
- fast tests stable on every change
- real-download smoke validation in CI
- heavier full real-download validation on schedule or before release

## Known Boundaries

- Shared global npm prefix is optimized for CLI tooling
- Packages with native addons or strong Node-version coupling may require reinstall for a specific runtime
- Upstream `fnm` behavior may be used as reference, but `nvc` remains independently maintained
