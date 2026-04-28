# nvc

[English](./README.md)

`nvc` 是一个跨平台 Node.js 版本管理器，强调可预测的 Shell 集成与可复现安装。

## 项目简介

- **状态**：持续维护中
- **定位**：参考 `fnm` 行为的独立项目，并非直接代码镜像
- **平台**：macOS、Linux、Windows
- **Shell 支持**：Bash、Zsh、Fish、PowerShell、Windows Cmd

## 核心特性

- 单文件二进制 CLI，启动快
- 支持 `.node-version`、`.nvmrc`、`package.json` engines 解析
- 多 Node 版本共享全局 npm 前缀
- 下载 Node 压缩包在激活前进行校验和验证
- 内置维护命令（`doctor`、`cache`、`prune`）

## 安装

### 脚本安装（macOS/Linux）

```sh
curl -o- https://raw.githubusercontent.com/wangsizhu0504/nvc/master/install.sh | bash
```

依赖：

- Linux：`curl`、`unzip`
- macOS（默认路径）：`curl`、`unzip`、`brew`（脚本默认使用 Homebrew）
- macOS 使用 `--force-install`：`curl`、`unzip`

常用脚本参数：

- `--install-dir <path>`
- `--skip-shell`
- `--force-install`（别名：`--force-no-brew`）
- `--release <tag|latest>`

### 预编译二进制

从 [GitHub Releases](https://github.com/wangsizhu0504/nvc/releases) 下载对应平台二进制，加入 `PATH` 后执行 Shell 初始化。

### Cargo

```sh
cargo install nvc
```

### Homebrew

```sh
brew install nvc
```

## Shell 初始化

将以下命令加入对应 Shell 启动文件。

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

## 常用命令

```sh
# 查看版本
nvc list-remote
nvc list

# 安装 / 切换
nvc install <version>
nvc use <version>
nvc current
nvc pin <version>

# 使用指定运行时执行命令
nvc exec --using=<version> node --version

# 诊断与清理
nvc doctor
nvc cache dir
nvc cache size --bytes
nvc cache clear
nvc prune --dry-run
nvc prune --all
```

## 发布流程

- 正式发布标签格式：`vX.Y.Z`
- Release 工作流触发方式：
  - 推送 `vX.Y.Z` 标签，或
  - 手动触发 `Release` 工作流并传入已存在标签
- 流程会执行质量检查、构建 macOS/Linux/Windows 二进制、生成 `checksums.txt` 并发布 GitHub Release 资产

## 开发

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

相关文档：

- [Testing Strategy](./docs/testing.md)
- [Release Checklist](./docs/release-checklist.md)
- [Maintainer Policy](./docs/maintainer-policy.md)
- [Support Policy](./docs/support-policy.md)

## 许可证

[MIT](./LICENSE) © 2024-PRESENT [Kriszu](https://github.com/wangsizhu0504)
