# Facedemo Optimization Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 完成日志实时链路、Tauri invoke 后端、桌面图片选择和构建文档的闭环改造，并保持 `dx serve` 降级可用。

**Architecture:** 先稳定 Tauri WebSocket 命令与 Cargo feature 分层，再把前端日志页和控制器接到新的命令语义上。图片选择走 Tauri 命令统一返回 Base64，`dx serve` 保留文本输入降级路径，避免非 Tauri 编译中断核心流程。

**Tech Stack:** Rust 2024, Dioxus 0.6, Tauri v2, Tokio, reqwest, rfd, base64

---

## Chunk 1: 日志链路与后端抽象

### Task 1: 先用测试固定新的日志语义

**Files:**
- Modify: `D:/code/facedemo/facedemo-rust/tests/backend_tests.rs`
- Modify: `D:/code/facedemo/facedemo-rust/tests/app_controller_tests.rs`

- [ ] **Step 1: 把 `backend_tests.rs` 中 `fetch_logs()` 断言改成新的启动语义**

```rust
let result = block_on(backend.fetch_logs());
assert!(result.is_ok());
```

- [ ] **Step 2: 把 `app_controller_tests.rs` 中 `refresh_logs_prepends_new_records` 改成 `start_log_listener_sets_status`**

```rust
block_on(controller.start_log_listener(&mut state)).expect("listener should start");
assert_eq!(state.status_message.as_deref(), Some("日志监听已启动"));
```

- [ ] **Step 3: 运行失败测试确认旧实现不匹配**

Run: `cargo test --test backend_tests --test app_controller_tests`
Expected: FAIL，原因是 `fetch_logs()` 返回类型和 `refresh_logs()` 旧语义不再匹配。

- [ ] **Step 4: 实现 `FaceBackend` 新的日志语义和控制器入口**

Files:
- `D:/code/facedemo/facedemo-rust/src/services/backend.rs`
- `D:/code/facedemo/facedemo-rust/src/state/app_controller.rs`

改动：
- `FaceBackend::fetch_logs()` 改为 `Result<(), BackendError>`
- 删除 `refresh_logs()`，新增 `start_log_listener()`
- `DemoBackend` 和 `HttpFaceBackend` 的 `fetch_logs()` 返回 `Ok(())`

- [ ] **Step 5: 运行测试确认通过**

Run: `cargo test --test backend_tests --test app_controller_tests`
Expected: PASS

### Task 2: 修复 Tauri WebSocket 重连缺陷

**Files:**
- Modify: `D:/code/facedemo/facedemo-rust/src-tauri/src/commands/websocket.rs`

- [ ] **Step 1: 先写或补一个最小测试覆盖 `JoinHandle::is_finished()` 相关逻辑**

说明：如果当前结构难以直接对命令函数写单测，就先提取一个纯函数用于判断“是否需要重建连接”，然后对纯函数写测试。

Suggested test:

```rust
assert!(should_restart_listener(Some(true)));
assert!(!should_restart_listener(Some(false)));
assert!(should_restart_listener(None));
```

- [ ] **Step 2: 运行对应测试，确认在提取逻辑前失败**

Run: `cargo test -p src-tauri`
Expected: FAIL，或新增测试无法编译直到辅助函数落地。

- [ ] **Step 3: 实现重连修复**

改动：
- 如果 `websocket_task` 为 `Some(handle)` 且 `handle.is_finished()`，先清空旧句柄
- 只有 `handle` 仍在运行时才直接 `return Ok(())`

- [ ] **Step 4: 运行 Tauri 测试确认通过**

Run: `cargo test -p src-tauri`
Expected: PASS

## Chunk 2: feature 门控与日志页接线

### Task 3: 先声明 feature，再实现 Tauri invoke 后端

**Files:**
- Modify: `D:/code/facedemo/facedemo-rust/Cargo.toml`
- Modify: `D:/code/facedemo/facedemo-rust/src-tauri/Cargo.toml`
- Modify: `D:/code/facedemo/facedemo-rust/src/services/backend.rs`

- [ ] **Step 1: 在两个 Cargo.toml 中声明 `tauri-backend` 和新增依赖**

改动：
- 前端 crate 增加 `[features]`
- Tauri crate 引用前端 crate 时开启 `features = ["tauri-backend"]`
- 前端增加 Tauri 绑定依赖
- Tauri crate 增加 `rfd`、`base64`

- [ ] **Step 2: 先运行基础编译，确认 feature 尚未接线完整时出现预期错误**

Run: `cargo check`
Expected: 可能 FAIL，原因是 backend 分支还未补齐。

- [ ] **Step 3: 在 `backend.rs` 中加入 `TauriInvokeBackend` 与 `AppBackend` 的 feature 分支**

要求：
- `fetch_users()` -> `get_user_list`
- `add_user()` -> `add_user`
- `delete_user()` -> `delete_user`
- `fetch_logs()` -> `start_websocket_listener`
- `from_env_or_demo()` 保持在 `backend.rs`

- [ ] **Step 4: 分别验证非 Tauri 与 Tauri 编译**

Run: `cargo check`
Expected: PASS

Run: `cargo check --manifest-path "D:/code/facedemo/facedemo-rust/src-tauri/Cargo.toml"`
Expected: PASS

### Task 4: 日志页事件订阅与 App 自动启动

**Files:**
- Modify: `D:/code/facedemo/facedemo-rust/src/app.rs`
- Modify: `D:/code/facedemo/facedemo-rust/src/pages/recognition_log.rs`

- [ ] **Step 1: 先补前端测试或最小编译断言**

说明：当前仓库没有组件测试基础设施，可用编译验证代替 UI 单测，但仍需先明确目标行为：
- `RecognitionLogPage` 新增 `on_new_logs`
- `App` 的 `on_refresh` 改接 `start_log_listener()`
- 自动启动逻辑仅在 `tauri-backend` 下编译

- [ ] **Step 2: 实现 `recognition_log.rs` 的事件订阅**

改动：
- 新增 `on_new_logs`
- `#[cfg(feature = "tauri-backend")]` 下订阅 `recognition-logs`
- 按设计把按钮文案改为“重新连接”

- [ ] **Step 3: 实现 `app.rs` 接线**

改动：
- `RecognitionLogPage.on_refresh` 调 `start_log_listener()`
- `RecognitionLogPage.on_new_logs` 调 `state.prepend_logs(logs)`
- 在切换到日志 tab 时通过 `use_effect` 自动触发一次 `start_log_listener()`

- [ ] **Step 4: 运行前端测试和编译验证**

Run: `cargo test --test app_controller_tests --test app_state_tests`
Expected: PASS

Run: `cargo check`
Expected: PASS

## Chunk 3: 图片选择与文档

### Task 5: 桌面图片选择命令和弹窗降级路径

**Files:**
- Modify: `D:/code/facedemo/facedemo-rust/src/components/user_info_modal.rs`
- Modify: `D:/code/facedemo/facedemo-rust/src/app.rs`
- Add: `D:/code/facedemo/facedemo-rust/src-tauri/src/commands/camera.rs`
- Modify: `D:/code/facedemo/facedemo-rust/src-tauri/src/commands/mod.rs`
- Modify: `D:/code/facedemo/facedemo-rust/src-tauri/src/lib.rs`

- [ ] **Step 1: 先写最小测试或断言覆盖错误路径**

优先补：
- `UserInfoModal` 新增 `on_error` 的编译接线
- `dx serve` 下仍保留 Base64 文本输入

如果组件单测不可行，则使用双编译链路验证这两个条件。

- [ ] **Step 2: 实现 `camera.rs`**

改动：
- `pick_image()` 桌面端使用 `rfd` + `base64`
- `take_photo()` 返回明确错误

- [ ] **Step 3: 注册命令**

改动：
- `commands/mod.rs` 导出 `camera`
- `src-tauri/src/lib.rs` 注册 `pick_image` 和 `take_photo`

- [ ] **Step 4: 改造 `user_info_modal.rs` 和 `app.rs`**

改动：
- `tauri-backend` 下显示“选择图片/拍照”入口与预览
- 非 `tauri-backend` 下保留 Base64 文本输入
- 新增 `on_error` 回调，并在 `app.rs` 中接到 `state.set_error(...)`

- [ ] **Step 5: 运行编译和测试**

Run: `cargo check`
Expected: PASS

Run: `cargo check --manifest-path "D:/code/facedemo/facedemo-rust/src-tauri/Cargo.toml"`
Expected: PASS

### Task 6: 补 BUILD 文档并做最终验证

**Files:**
- Add: `D:/code/facedemo/facedemo-rust/BUILD.md`

- [ ] **Step 1: 根据仓库真实结构写 `BUILD.md`**

内容必须覆盖：
- 前置依赖
- 桌面端构建
- Android 端构建
- 签名配置
- `.env` 说明

- [ ] **Step 2: 运行最终验证**

Run: `cargo test --test backend_tests --test app_controller_tests --test app_state_tests`
Expected: PASS

Run: `cargo test --manifest-path "D:/code/facedemo/facedemo-rust/src-tauri/Cargo.toml"`
Expected: PASS

Run: `cargo check`
Expected: PASS

Run: `cargo check --manifest-path "D:/code/facedemo/facedemo-rust/src-tauri/Cargo.toml"`
Expected: PASS

- [ ] **Step 3: 记录未验证项**

如果没有真机、没有运行中的 WebSocket 服务、没有实际弹出文件对话框环境，需要在交付说明中明确列出这些人工验证缺口。
