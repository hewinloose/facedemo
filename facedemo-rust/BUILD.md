# FaceDemo 构建说明

## 前置依赖

- Rust 工具链：建议使用稳定版，且至少支持当前仓库依赖要求。
- WebAssembly 目标：
  - `rustup target add wasm32-unknown-unknown`
- Dioxus CLI：
  - `cargo install cargo-binstall`
  - `cargo binstall dioxus-cli`
- Tauri CLI：
  - `cargo install tauri-cli --version "^2"`
- Android 构建依赖：
  - Android SDK
  - Android NDK
  - JDK

## 当前前端架构

本项目当前采用 `Tauri + Dioxus Web`：

- Dioxus 前端通过 `dx build --platform web` 构建到 `dist/`
- Tauri 通过 [tauri.conf.json](/D:/code/facedemo/facedemo-rust/src-tauri/tauri.conf.json) 加载 `../dist`
- Tauri 构建时会调用：
  - `beforeDevCommand`: `dx serve --platform web -- --features tauri-backend`
  - `beforeBuildCommand`: `dx build --release --platform web -- --features tauri-backend`

非 Tauri 的 `dx serve` 调试模式不启用 `tauri-backend`，前端会自动降级到 Demo 数据和 Base64 文本输入。

## 桌面端开发

在前端目录执行：

```bash
cd facedemo-rust
dx serve --platform web
```

如果需要联调 Tauri 命令层，执行：

```bash
cd facedemo-rust/src-tauri
cargo tauri dev
```

要求：

- 已安装 `dx`
- 已安装 `wasm32-unknown-unknown`
- `cargo tauri dev` 能找到 `dx`

## 桌面端构建

```bash
cd facedemo-rust/src-tauri
cargo tauri build
```

默认产物目录：

- `facedemo-rust/src-tauri/target/release/bundle/`

## Android 端构建

```bash
cd facedemo-rust/src-tauri
cargo tauri android build
```

常见产物目录：

- `facedemo-rust/src-tauri/gen/android/app/build/outputs/apk/`

## 签名配置

签名文件建议不提交到仓库，使用本地路径或 CI Secret 注入。

建议做法：

- keystore 文件放在仓库外，或放在忽略列表覆盖的本地路径
- 密码使用环境变量传入

示例环境变量：

```bash
set TAURI_SIGNING_PRIVATE_KEY=...
set TAURI_SIGNING_PRIVATE_KEY_PASSWORD=...
```

如果使用 Android keystore，请同时维护：

- keystore 路径
- alias
- store password
- key password

避免把这些信息硬编码到配置文件里。

## `.env` 说明

桌面原生 HTTP 调试链路仍支持读取 `.env`，常用变量包括：

- `BAIDU_CLIENT_ID`
- `BAIDU_CLIENT_SECRET`
- `BAIDU_GROUP_ID`
- `WS_SERVER_URL`

说明：

- Tauri Web 前端构建产物本身不读取 `.env`
- `dx serve` 的 Web 调试模式默认走 Demo 数据
- `cargo tauri dev/build` 下，HTTP 请求和 WebSocket 连接由 Tauri Rust 后端读取环境变量并执行

## 验证建议

至少完成以下验证：

1. `cargo check`
2. `cargo test --test backend_tests --test app_controller_tests --test app_state_tests`
3. `cd src-tauri && cargo check`
4. `cd src-tauri && cargo test`
5. `rustup target list --installed` 中包含 `wasm32-unknown-unknown`

如果第 5 条不满足，无法本地验证 Dioxus Web 目标编译。
