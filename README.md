<h1 align="center">
  Node Version Control Tool
</h1>

> A self-contained version control tool for Node.JS, fork from [fnm](https://github.com/Schniz/fnm)

## Features

🌎 Cross-platform support (macOS, Windows, Linux)

✨ Single file, easy installation, instant startup

🚀 Built with speed in mind

📂 Works with `.node-version` and `.nvmrc` files

## Installation

### Using a script (macOS/Linux)

For `bash`, `zsh` and `fish` shells, there's an [automatic installation script](./install.sh).

First ensure that `curl` and `unzip` are already installed on you operating system. Then execute:

```sh
curl -o- https://raw.githubusercontent.com/wangsizhu0504/nvc/master/install.sh | bash
```

#### Upgrade

On macOS, it is as simple as `brew upgrade nvc`.

On other operating systems, upgrading `nvc` is almost the same as installing it. To prevent duplication in your shell config file add `--skip-shell` to install command.

#### Parameters

`--install-dir`

Set a custom directory for nvc to be installed. The default is `$XDG_DATA_HOME/nvc` (if `$XDG_DATA_HOME` is not defined it falls back to `$HOME/.local/share/nvc` on linux and `$HOME/Library/Application Support/nvc` on MacOS).

`--skip-shell`

Skip appending shell specific loader to shell config file, based on the current user shell, defined in `$SHELL`. e.g. for Bash, `$HOME/.bashrc`. `$HOME/.zshrc` for Zsh. For Fish - `$HOME/.config/fish/conf.d/nvc.fish`

`--force-install`

macOS installations using the installation script are deprecated in favor of the Homebrew formula, but this forces the script to install using it anyway.

Example:

```sh
curl -o- https://raw.githubusercontent.com/wangsizhu0504/nvc/master/install.sh | bash -s -- --install-dir "./.nvc" --skip-shell
```

### Manually

#### Using Homebrew (macOS/Linux)

```sh
brew install nvc
```

Then, [set up your shell for nvc](#shell-setup)

#### Using Cargo (Linux/macOS/Windows)

```sh
cargo install nvc
```

Then, [set up your shell for nvc](#shell-setup)

#### Using a release binary (Linux/macOS/Windows)

- Download the [latest release binary](https://github.com/wangsizhu0504/nvc/releases) for your system
- Make it available globally on `PATH` environment variable
- [Set up your shell for nvc](#shell-setup)

### Removing

To remove nvc (😢), just delete the `.nvc` folder in your home directory. You should also edit your shell configuration to remove any references to nvc (ie. read [Shell Setup](#shell-setup), and do the opposite).

## Completions

nvc ships its completions with the binary:

```sh
nvc completions --shell <SHELL>
```

Where `<SHELL>` can be one of the supported shells:

- `bash`
- `zsh`
- `fish`
- `power-shell`

Please follow your shell instructions to install them.

### Shell Setup

Environment variables need to be setup before you can start using nvc.
This is done by evaluating the output of `nvc env`.
To automatically run `nvc use` when a directory contains a `.node-version` or `.nvmrc` file, add the `--use-on-cd` option to your shell setup.

Adding a `.node-version` to your project is as simple as:

```bash
$ node --version
v14.18.3
$ node --version > .node-version
```

Check out the following guides for the shell you use:

#### Bash

Add the following to your `.bashrc` profile:

```bash
eval "$(nvc env --use-on-cd)"
```

#### Zsh

Add the following to your `.zshrc` profile:

```zsh
eval "$(nvc env --use-on-cd)"
```

#### Fish shell

Create `~/.config/fish/conf.d/nvc.fish` add this line to it:

```fish
nvc env --use-on-cd | source
```

#### PowerShell

Add the following to the end of your profile file:

```powershell
nvc env --use-on-cd | Out-String | Invoke-Expression
```

- For macOS/Linux, the profile is located at `~/.config/powershell/Microsoft.PowerShell_profile.ps1`
- On Windows to edit your profile you can run this in a PowerShell
  ```powershell
  notepad $profile
  ```
#### Windows Command Prompt aka Batch aka WinCMD

nvc is also supported but is not entirely covered. [You can set up a startup script](https://superuser.com/a/144348) and append the following line:

```batch
FOR /f "tokens=*" %i IN ('nvc env --use-on-cd') DO CALL %i
```

⚠️ If you get the error `i was unexpected at this time`, please make a .cmd file as suggested by the first step in the Usage with Cmder secton add it's path to the `AutoRun` registry key.

#### Usage with Cmder

Usage is very similar to the normal WinCMD install, apart for a few tweaks to allow being called from the cmder startup script. The example **assumes** that the `CMDER_ROOT` environment variable is **set** to the **root directory** of your Cmder installation.
Then you can do something like this:

- Make a .cmd file to invoke it

```batch
:: %CMDER_ROOT%\bin\nvc_init.cmd
@echo off
FOR /f "tokens=*" %%z IN ('nvc env --use-on-cd') DO CALL %%z
```

- Add it to the startup script

```batch
:: %CMDER_ROOT%\config\user_profile.cmd
call "%CMDER_ROOT%\bin\nvc_init.cmd"
```

You can replace `%CMDER_ROOT%` with any other convenient path too.

## Usage

[For extended usage documentation, see Fnm's command](https://github.com/Schniz/fnm/blob/master/docs/commands.md)


### Developing:

```sh
# Install Rust
git clone https://github.com/wangsizhu0504/nvc.git
cd nvc/
cargo build
```

### Running Binary:

```sh
cargo run -- --help # Will behave like `nvc --help`
```

### Running Tests:

```sh
cargo test
```

## Thanks

This project is based on [Fnm](https://github.com/Schniz/fnm).

## License

[MIT](./LICENSE) License &copy; 2024-PRESENT [Kriszu](https://github.com/wangsizhu0504)