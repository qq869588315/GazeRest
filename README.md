# GazeRest

GazeRest 是一个轻量、离线、低打扰的 20-20-20 护眼桌面助手。它常驻系统托盘，在用户连续用屏一段时间后，提醒用户短暂把视线移向远处。

English README: [README_en.md](README_en.md)

## 下载安装包

仓库内已附带 Windows 1.0.0 安装包，位置在 `release/v1.0.0/`。

- 推荐普通用户使用：[GazeRest_1.0.0_x64-setup.exe](release/v1.0.0/GazeRest_1.0.0_x64-setup.exe)
- 如需 MSI 安装包，可使用：[GazeRest_1.0.0_x64_en-US.msi](release/v1.0.0/GazeRest_1.0.0_x64_en-US.msi)

一般测试或分发给朋友时，直接发送 `GazeRest_1.0.0_x64-setup.exe` 即可。

## 核心功能

- 20-20-20 护眼提醒：默认连续用屏 20 分钟后提醒休息。
- 4 档提醒等级：从仅状态提示到沉浸式提醒，用户可以按打扰容忍度选择。
- 20 秒休息倒计时：支持极简数字、呼吸动效、轻引导模式。
- 建议观看距离计算：输入或自动获取显示器宽高后，按对角线估算建议距离。
- 系统托盘常驻：关闭主窗口后仍可在后台运行。
- 本地持久化：设置、提醒记录、休息记录保存在本机 SQLite。
- 双语界面：支持简体中文和英文。

## 技术栈

- 桌面壳：Tauri 2
- 后端：Rust
- 前端：React 19 + TypeScript + Vite
- 样式：CSS Modules + CSS 变量
- 国际化：i18next + react-i18next
- 本地数据库：SQLite + rusqlite
- 系统能力：Tauri tray、single-instance、autostart、positioner、logging 插件

## 架构说明

GazeRest 采用“前端只做界面，Rust 负责核心逻辑”的结构，避免提醒调度散落在 UI 里。

### 前端

前端负责展示主面板、设置页、测距页、提醒窗和休息倒计时窗。

- `src/ui`：窗口、组件、图标和样式。
- `src/modules`：前端桥接、格式化、声音预览、窗口控制等轻逻辑。
- `src/i18n`：中文和英文文案。
- `src/types`：前后端共享的 TypeScript 类型。

主窗口内部通过视图状态切换 `panel / settings / distance`，没有复杂路由。提醒窗和休息窗由 Tauri 使用独立窗口打开。

### Rust 后端

Rust 负责应用生命周期、托盘、窗口管理、提醒状态机、系统空闲检测、全屏检测、休息倒计时、SQLite 持久化和日志。

- `src-tauri/src/commands.rs`：暴露给前端调用的 Tauri 命令。
- `src-tauri/src/scheduler.rs`：后台轮询、连续用屏计时、提醒触发和休息倒计时。
- `src-tauri/src/windows.rs`：主窗口、提醒窗口、休息窗口的创建、定位和显示。
- `src-tauri/src/db.rs`：SQLite 读写和统计聚合。
- `src-tauri/src/models.rs`：设置、运行状态、提醒事件和休息会话类型。

### 数据与隐私

应用完全离线运行，不上传数据，不记录屏幕内容，不记录按键内容，也不请求摄像头或麦克风权限。数据库仅用于保存本地设置、提醒事件和休息会话。

## 开发

```bash
npm install
npm run tauri:dev
```

## 本地检查

```bash
npm run test:run
npm run build
cd src-tauri
cargo check
```

## 打包发布

生成正式安装包：

```bash
npm run tauri:build
```

Windows 构建产物通常在：

```text
src-tauri/target/release/bundle/
```

当前仓库内保留的 1.0.0 安装包副本在：

```text
release/v1.0.0/
```

如果只想快速验证 release exe，不生成安装包：

```bash
npx tauri build --no-bundle
```

快速构建生成的 exe 在：

```text
src-tauri/target/release/app.exe
```

## 1.0 发布检查清单

- 确认版本号为 `1.0.0`。
- 运行 `npm run test:run`。
- 运行 `npm run build`。
- 运行 `cargo check`。
- 运行 `npm run tauri:build` 生成安装包。
- 在干净 Windows 环境中安装、启动、托盘常驻、退出和卸载各测试一次。
- 将安装包上传到 GitHub Releases，并附上简短更新说明。
