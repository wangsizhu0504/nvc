# AGENT Guide

## 项目定位

- 这是一个 Rust 2021 的单二进制 CLI 项目，不是 workspace，也不是 library-first 结构。
- 项目目标是管理本机 Node.js 版本，整体架构沿用 `fnm` 一类工具的思路，强调跨平台、单文件分发和命令行可用性。
- 入口在 `src/main.rs`，主流程很短：初始化日志，解析 CLI，再把执行交给具体 subcommand。

## 目录与职责

- `src/cli.rs`：定义顶层 CLI、subcommand 枚举，以及分发逻辑。
- `src/commands/*.rs`：每个命令一个文件，命令参数与业务入口都放在这里。
- `src/commands/command.rs`：统一的 `Command` trait，负责 `apply -> call -> 错误输出并退出` 这条执行链。
- `src/shell/*`：shell 相关逻辑，包括环境变量输出、shell 推断、Windows CMD 兼容。
- `src/archive/*`、`src/downloader.rs`、`src/http.rs`：下载、解压、HTTP 请求。
- `src/version.rs`、`src/user_version.rs`、`src/user_version_reader.rs`、`src/version_files.rs`：版本解析、推断和版本文件读取。
- `src/fs.rs`、`src/directories.rs`、`src/path_ext.rs`：文件系统和目录策略，含跨平台差异。

## 新增或修改命令时的规则

1. 新命令优先按现有模式新增到 `src/commands/<name>.rs`。
2. 命令参数结构体使用 `#[derive(clap::Parser, Debug)]`。
3. 命令执行统一实现 `Command` trait，把业务逻辑放进 `apply(&NvcConfig) -> Result<(), Error>`。
4. 新命令接入时同时更新：
   - `src/commands/mod.rs`
   - `src/cli.rs` 的 `SubCommand`
   - `src/cli.rs` 的 `match` 分发
5. 不要绕过 `Command::call` 自己处理统一退出逻辑，除非确实需要改变整个命令执行模型。

## 代码风格与实现偏好

- 优先沿用现有的扁平模块组织，不为了“更工程化”再包一层 service、manager、adapter。
- 版本相关逻辑优先复用已有类型：`Version`、`UserVersion`、`UserVersionReader`、`LtsType`、`Arch`，不要再造一套解析模型。
- 错误类型默认使用 `thiserror` 的局部枚举，贴近命令或模块本身。
- `anyhow` 在仓库里已有使用，但主要出现在 shell 这类字符串拼装边界；常规命令逻辑仍优先显式错误类型。
- 对用户可见的普通输出，优先使用 `outln!(config, Info/Error, ...)`，这样能遵守 `NVC_LOGLEVEL`。
- 不要因为看到现有代码里有 `unwrap` / `expect` 就顺手大面积清理。仓库当前接受少量启动期、测试期和明显不该失败路径上的 `unwrap`。只有在你正在修的真实失败链路上，才把它收敛成可恢复错误。

## 跨平台修改规则

- 这是一个明确的跨平台项目，不能只按当前机器行为改。
- 遇到平台差异，优先落在已有边界内处理：
  - 符号链接与删除：`src/fs.rs`
  - shell 行为差异：`src/shell/*`
  - 路径与目录策略：`src/directories.rs`
  - 编译条件分支：`#[cfg(unix)]` / `#[cfg(windows)]`
- 不要把零散的 `cfg!()` 判断扩散到业务层；如果问题只属于某个平台或某个 shell，就在对应模块修。
- Windows 相关改动要留意 `build.rs` 和 `nvc.manifest.rc` 的存在，不要误删发布链路需要的资源编译步骤。

## 调试时先看什么

- 版本解析问题：先看 `Version` / `UserVersion` / `UserVersionReader` 的真实输入与输出。
- 自动切换问题：先看 `NVC_MULTISHELL_PATH`、实际 symlink/junction、`PATH` 里是否包含 multishell 路径。
- 下载或远端版本问题：先看 `remote_node_index::list` 和 `http.rs` 的真实响应，不先加新抽象。
- shell 初始化问题：先看 `nvc env` 生成的真实脚本，再看 `src/shell/*` 的实现，不先猜测终端行为。
- 解压或安装目录问题：先看真实目录布局，当前安装结构约定是 `.../<version>/installation`。

## 测试与验证约定

- 默认验证命令是 `cargo test`。
- 现有测试以模块内联 `#[cfg(test)]` 为主，不是单独的 integration test 目录；新增测试优先贴近被测模块。
- 断言风格沿用 `pretty_assertions::assert_eq`。
- 需要日志或更接近真实行为的测试，仓库已经使用 `test_log::test` 和 `duct`，可按现有方式继续写。
- 一部分测试会访问网络或下载真实 Node 分发包，例如：
  - `remote_node_index`
  - `downloader`
  - `commands::install`
- 因此不要把测试当成纯离线单元测试；改动相关逻辑时，优先做定向测试，再决定是否跑全量。

## 关于格式化与 CI

- 当前仓库不是 `cargo fmt --check` 全绿状态，存在既有格式漂移。
- 不要为了顺手“清仓”做全仓格式化，避免把无关 diff 混进功能修复。
- 如果你改了某个文件，优先只保证改动部分可读、风格一致；需要格式化时，控制在受影响文件范围内。
- `.github/workflows/rust.yml` 当前主要做 Windows、macOS、Linux 的 release build，不替代本地测试验证。

## 改动边界

- 这是 CLI 工具，不要引入前端式 DTO、normalize、view model 一类中间层。
- 不要把简单的命令流拆成多级抽象，只因为“看起来更统一”。
- 修 bug 时只改真正负责该行为的层：
  - CLI 参数问题改 `cli.rs` / 对应 command
  - 版本推断问题改版本读取与解析模块
  - shell 问题改 shell 模块
  - 下载解压问题改 downloader/archive/http

## 交付要求

- 最终说明至少交代：
  - 根因是什么
  - 改动落在哪一层
  - 为什么这样改能解决问题
  - 实际跑了哪些验证
- 如果验证受网络、平台或外部环境限制，要明确说清楚，不默认当作“已验证”。
