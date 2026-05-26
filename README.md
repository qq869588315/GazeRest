# GazeRest

GazeRest 是一个轻量、离线、低打扰的 20-20-20 护眼桌面助手。它常驻系统托盘，在你连续用屏一段时间后提醒你把视线移向远处，并用 20 秒倒计时帮助完成一次短休息。

English README: [README_en.md](README_en.md)

## 核心功能

- 20-20-20 护眼提醒：默认连续用屏 20 分钟后提醒休息，正式可选间隔为 20 / 30 / 40 / 50 / 60 分钟。
- 四级提醒强度：
  - Level 0：不弹窗，只更新主面板和托盘状态。
  - Level 1：右下角轻提醒，不抢焦点，适合日常办公。
  - Level 2：居中提醒卡片，提供跳过、稍后和开始休息。
  - Level 3：全屏沉浸提醒，内容居中，适合必须休息的场景。
- 20 秒休息倒计时：每次休息都会创建新的休息会话，倒计时从完整时长开始。
- 用屏统计：区分“已连续用屏时间”“今日用屏”和“今日最长连续用屏”。
- 观看距离计算：根据显示器宽高估算建议观看距离。
- 系统托盘常驻：关闭主窗口后仍可在后台运行，并用不同托盘状态区分运行、暂停、延后、提醒和休息中。
- 双语界面：支持简体中文和英文。
- 本地持久化：设置、提醒事件和休息记录保存在本机 SQLite 数据库。

## 隐私说明

GazeRest 完全离线运行，不上传数据，不记录屏幕内容，不记录按键内容，也不请求摄像头或麦克风权限。所有设置、提醒记录和休息记录仅保存在本机。

## 技术栈

- 桌面壳：Tauri 2
- 后端：Rust
- 前端：React 19 + TypeScript + Vite
- 样式：CSS Modules
- 国际化：i18next + react-i18next
- 本地数据库：SQLite + rusqlite
- 系统能力：tray、single-instance、autostart、positioner、logging

## 架构说明

GazeRest 采用“前端负责展示，Rust 负责核心状态机”的结构，避免提醒调度和休息倒计时散落在 UI 里。

- `src/ui`：主面板、设置页、测距页、提醒窗、休息窗和样式。
- `src/modules`：前端桥接、格式化、声音预览和窗口辅助逻辑。
- `src/i18n`：中文和英文文案。
- `src/types`：前后端共享的 TypeScript 类型。
- `src-tauri/src/runtime_service.rs`：提醒、休息、暂停、延后、跳过和启动恢复的核心状态机。
- `src-tauri/src/windows.rs`：主窗口、提醒窗口和休息窗口的创建、定位和销毁。
- `src-tauri/src/tray_service.rs`：托盘图标状态和提示文案。
- `src-tauri/src/sound_service.rs`：提醒音效。
- `src-tauri/src/db.rs`：SQLite 读写、迁移和统计聚合。

提醒窗和休息窗采用按需创建、用完关闭的方式，避免隐藏窗口复用导致的旧状态残留。

## 开发

```bash
npm install
npm run tauri:dev
```

## 本地检查

```bash
npm run lint
npm run test:run
npm run build
cd src-tauri
cargo check
cargo test
```

## 打包发布

生成正式安装包：

```bash
npm run tauri:build
```

Windows 构建产物：

```text
src-tauri/target/release/app.exe
src-tauri/target/release/bundle/nsis/GazeRest_1.0.2_x64-setup.exe
src-tauri/target/release/bundle/msi/GazeRest_1.0.2_x64_en-US.msi
```

如果只想快速验证 release exe，不生成安装包：

```bash
npx tauri build --no-bundle
```

仓库中的 `release/v1.0.0/` 是历史发布副本；正式发布请以本次 `npm run tauri:build` 生成的安装包为准。

## 1.0.2 发布信息

- 版本号：`1.0.2`
- 发布日期：2026-05-26
- 当前发布平台：Windows x64
- 推荐分发包：`GazeRest_1.0.2_x64-setup.exe`
- 可选 MSI 包：`GazeRest_1.0.2_x64_en-US.msi`

## 1.0.2 更新内容

- 修复重新开机或重新打开 App 后继续沿用上次临时状态的问题。
- 重新启动 App 时会清理待处理提醒、休息中、稍后、暂停和本轮连续用屏时间，开启新的提醒周期。
- 保留“今日用屏”和“今日最长连续用屏”的当天累计；跨本地日期后仍会自动归零。
- 退出或重启时会把未完成的休息会话标记为 interrupted，避免下次启动继续旧倒计时。

## 1.0.1 更新内容

- 修复旧本地数据中可能残留 1 分钟测试提醒间隔的问题，启动时会自动恢复为正式的 20 分钟。
- 增加提醒间隔合法性保护，只允许正式选项 20 / 30 / 40 / 50 / 60 分钟生效。
- 保留 1.0.0 的稳定性修复：提醒窗、休息倒计时、托盘状态、音效和今日用屏统计。

## 发布检查清单

- 确认正式设置页不包含测试用 1 分钟提醒间隔。
- 运行 `npm run lint`。
- 运行 `npm run test:run`。
- 运行 `npm run build`。
- 运行 `cargo check` 和 `cargo test`。
- 运行 `npm run tauri:build` 生成安装包。
- 在 Windows 环境中验证启动、托盘常驻、提醒弹窗、休息倒计时、声音、退出和卸载。
