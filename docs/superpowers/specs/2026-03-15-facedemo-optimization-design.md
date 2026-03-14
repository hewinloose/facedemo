# Facedemo 改造设计

**日期**：2026-03-15

**目标**

基于 [CODE_TASKS.md](/D:/code/facedemo/CODE_TASKS.md) 完成 `T1` 到 `T5` 的项目优化，优先修复日志实时链路与 Tauri 后端接入，再补齐桌面图片选择和构建文档，保持实现简单、可编译、可验证。

## 1. 范围

本次实现包含以下范围：

1. 日志页订阅 Tauri `recognition-logs` 事件并实时更新列表。
2. 修正日志命令语义，将“获取日志列表”调整为“启动日志流”。
3. 用 Cargo feature 在编译期门控 Tauri invoke 后端。
4. 在添加用户弹窗中接入桌面图片选择，移动端只预留接口。
5. 新增构建与打包说明文档。

本次不包含以下范围：

1. 移动端真机拍照能力落地。
2. WebSocket 快照拉取协议扩展。
3. 与本次任务无关的 UI 重构或状态管理重写。

## 2. 现状问题

### 2.1 日志链路

- 前端日志页目前只有手动刷新，没有订阅 `recognition-logs`。
- `fetch_logs()` 始终返回示例种子数据，无法接入真实 WebSocket 数据。
- `refresh_logs()` 把“拉取列表”和“写入状态”耦合在一起，与事件流模式冲突。

### 2.2 后端接入方式

- 当前前端直连 HTTP，无法覆盖 Tauri 桌面/移动端必须通过命令层调用的场景。
- 不能依赖运行时判断是否存在 Tauri IPC，必须用编译期 feature 分流。

### 2.3 图片输入体验

- 添加用户只能手工粘贴 Base64，体验差且容易出错。
- 桌面端缺少文件选择命令，移动端拍照能力也没有预留。

### 2.4 交付文档

- 仓库缺少从源码到桌面包、APK 的完整构建说明。

## 3. 方案选择

### 推荐方案：保留 `fetch_logs()` 名称，但将其职责收窄为“启动监听”

后端能力分成两类：

- 请求型接口：`fetch_users`、`add_user`、`delete_user`
- 日志启动接口：`fetch_logs`

`fetch_logs()` 不再返回日志列表，而是仅用于触发日志监听：

1. 前端触发 `fetch_logs()`
2. Tauri 连接 WebSocket
3. Tauri `emit("recognition-logs", logs)`
4. 前端页面监听事件并写入 `AppState`

选择这个方案的原因：

- 与当前任务单和现有代码命名兼容，减少跨文件改名带来的扰动。
- 通过控制器层新增语义化入口 `start_log_listener()`，可以把“接口名兼容”与“UI 语义清晰”同时满足。
- 与 `CODE_TASKS.md` 的“先修命令，再自动启动监听”要求一致。

不采用的方案：

- 将 `fetch_logs()` 全量重命名为 `start_log_stream()`：语义更干净，但会引入额外重构面，本次没有必要。
- 同时做“启动监听 + 拉一次快照”：需要新增协议，超出当前范围。

## 4. 架构设计

### 4.1 前端后端抽象

`FaceBackend` 保留现有请求型接口，并保留 `fetch_logs()` 名称，但调整返回语义：

- `fetch_users()`
- `add_user(new_user)`
- `delete_user(user_id)`
- `fetch_logs() -> Result<(), BackendError>`

编译期按 feature 分流：

- 不带 `tauri-backend`：
  - `HttpFaceBackend` 或 `DemoBackend`
- 带 `tauri-backend`：
  - `TauriInvokeBackend`

`AppBackend::from_env_or_demo()` 继续定义在 `services/backend.rs`，并在这里做编译期切分：

- `tauri-backend` 下直接返回 `Tauri`
- 非 `tauri-backend` 下维持现有 `Http -> Demo` 回退逻辑

### 4.2 日志流数据流

日志页的数据流如下：

1. 进入“识别日志”页。
2. 页面挂载后订阅 `recognition-logs` 事件。
3. `App` 在切换到日志 tab 时通过 `use_effect` 自动触发一次 `start_log_listener()`。
4. `AppController::start_log_listener()` 调用 `backend.fetch_logs()`。
5. Tauri 命令建立 WebSocket 连接并持续派发日志事件。
6. 页面收到 `Vec<RecognitionLogEntry>` 后调用 `on_new_logs`。
7. `App` 将日志追加到 `AppState.logs` 顶部。

“刷新”按钮的语义调整为：

- 重新触发日志监听命令
- 作为连接失败后的重试入口

### 4.3 图片选择数据流

添加用户弹窗的数据流如下：

1. 用户点击“从文件选择”。
2. 前端调用 `invoke("pick_image")`。
3. Tauri 桌面端通过 `rfd` 选择图片文件。
4. Tauri 读取文件并转为 Base64 字符串返回。
5. 前端写入 `user_draft.image_base64`。
6. 弹窗显示预览图，并继续复用现有提交流程。

`take_photo` 本次只做接口预留：

- 前端保留按钮或入口
- Tauri 命令返回明确的“当前平台暂不支持拍照”错误
- `invoke_handler` 中仍需注册该命令，避免前端调用时报“未注册命令”

## 5. 文件级改动

### 5.1 前端 crate

#### `facedemo-rust/Cargo.toml`

- 新增：
  - `[features]`
  - `default = []`
  - `tauri-backend = []`
- 补充前端在 `tauri-backend` 下所需依赖，例如 Tauri 事件与 invoke 绑定库。

#### `facedemo-rust/src/services/backend.rs`

- 保留 `FaceBackend::fetch_logs()` 方法名，但将返回类型改为 `Result<(), BackendError>`。
- 保留 `DemoBackend`、`HttpFaceBackend` 的用户管理实现。
- 为 `DemoBackend` 提供一个最小 `fetch_logs()` 实现：
  - 可返回成功但不做任何事，或按现有示例状态保持静默。
- 在 `#[cfg(feature = "tauri-backend")]` 下新增 `TauriInvokeBackend`：
  - `fetch_users()` -> `invoke("get_user_list", ...)`
  - `add_user()` -> `invoke("add_user", ...)`
  - `delete_user()` -> `invoke("delete_user", ...)`
  - `fetch_logs()` -> `invoke("start_websocket_listener", ...)`
- 在 `AppBackend` 中增加 `Tauri` variant，并补齐对应 match 分支。
- `AppBackend::from_env_or_demo()` 的 `#[cfg]` 门控放在这里，`app.rs` 只负责调用。

#### `facedemo-rust/src/state/app_controller.rs`

- 保留用户相关逻辑。
- 删除现有 `refresh_logs()`，统一由 `start_log_listener()` 取代日志相关入口。
- tab 切换自动启动和“重新连接”按钮都调用同一个 `start_log_listener()`，避免重复实现。
- 不在控制器内部接收日志数组，也不直接写入 `state.logs`。
- 现有引用 `refresh_logs()` 的调用点和测试一并改名为 `start_log_listener()`。

#### `facedemo-rust/src/pages/recognition_log.rs`

- 新增 `on_new_logs` 回调参数。
- 页面挂载时订阅 `recognition-logs`。
- 不在页面内部触发自动启动。
- 保留“查看图片”和“重新连接”按钮。

#### `facedemo-rust/src/app.rs`

- 在切换到 `AppTab::RecognitionLog` 时，通过 `use_effect` 自动触发一次 `start_log_listener()`。
- 给 `RecognitionLogPage` 传入 `on_new_logs`。
- 给日志页的 `on_refresh` 回调接 `start_log_listener()`，按钮语义固定为“重新连接”。
- 在回调中调用 `state.prepend_logs(logs)`。
- 给 `UserInfoModal` 新增 `on_error` 回调绑定，统一写入 `AppState::set_error()`。
- 维持现有的 tab 切换、模态框和图片查看逻辑。

#### `facedemo-rust/src/components/user_info_modal.rs`

- 在 `tauri-backend` 下移除 Base64 文本输入框作为主交互，改为“从文件选择”入口。
- 在非 `tauri-backend` 下保留 Base64 文本输入作为降级路径，保证 `dx serve` 仍可手动添加用户。
- 预留“拍照”入口，但桌面下只展示提示或错误。
- 增加预览区域，直接展示当前 `image_base64`。
- 新增 `on_error: EventHandler<String>`，用于把 `pick_image` / `take_photo` 的失败信息上抛给父组件。
- 保持表单字段校验逻辑最小化，不引入新状态结构。

#### `facedemo-rust/src/state/app_state.rs`

- 保留 `user_draft.image_base64` 作为单一图片数据来源。
- 继续使用 `prepend_logs()` 追加实时日志。
- 如有需要，补充最小辅助方法，但不扩展状态层职责。

### 5.2 Tauri crate

#### `facedemo-rust/src-tauri/Cargo.toml`

- 依赖前端 crate 时启用 `tauri-backend` feature。
- 新增：
  - `rfd`
  - `base64`

#### `facedemo-rust/src-tauri/src/commands/websocket.rs`

- 先修正命令行为，再让前端自动调用。
- 保证命令在已启动、已断开、首次启动三种状态下行为明确。
- 继续保持单实例监听，避免重复连接。
- 如果当前任务句柄已结束，应允许重新启动。

#### `facedemo-rust/src-tauri/src/commands/camera.rs`

- 新增 `pick_image()`：
  - 桌面端选择文件
  - 读取字节
  - Base64 编码
  - 返回字符串
- 新增 `take_photo()`：
  - 当前先返回“未实现/当前平台不支持”

#### `facedemo-rust/src-tauri/src/commands/mod.rs`

- 注册 `camera` 模块。

#### `facedemo-rust/src-tauri/src/lib.rs`

- 在 `invoke_handler` 中注册 `pick_image`、`take_photo`。

## 6. 错误处理

### 6.1 日志流

- `fetch_logs()` 成功仅表示命令触发成功，不代表已经收到日志。
- 命令调用失败时，前端通过 `state.set_error()` 显示错误信息。
- WebSocket 连接失败、消息解析失败等细节由 Tauri 侧记录日志。
- 前端重试入口统一通过“重新连接”按钮和 tab 切换自动触发。

### 6.2 图片选择

- 用户取消选择文件时，返回明确错误或空结果，不覆盖已有图片。
- 文件读取失败、编码失败时，只更新错误提示。
- 错误提示路径固定为：`UserInfoModal` 调用 `on_error(String)`，由 `app.rs` 转成 `state.set_error(...)`。
- 不引入额外文件路径缓存，避免状态复杂化。

### 6.3 编译兼容

- 所有 Tauri 相关前端代码必须放在 `tauri-backend` feature 门控下。
- 非 Tauri 编译不得直接引用 `invoke` 或事件监听 API。
- `dx serve` 下通过保留 Base64 文本输入实现功能降级，不要求文件选择命令可用。

## 7. 测试与验证策略

遵循最小必要覆盖，只验证本次改动引入的新行为。

### 7.1 前端单元测试

优先覆盖纯逻辑：

- `AppState::prepend_logs()` 的追加顺序
- `AppController::start_log_listener()` 的成功/失败状态消息
- `AppBackend` 在不同 feature 下的初始分支行为
- `UserInfoModal` 图片选择失败时会通过 `on_error` 上抛错误

### 7.2 编译验证

必须至少验证两条编译链路：

1. 非 Tauri feature：
   - `cargo check`
2. Tauri crate：
   - `cargo check -p src-tauri` 或对应 Tauri crate 检查命令

如果仓库已存在测试命令，额外执行相关测试。

### 7.3 行为验证

人工验证至少覆盖：

1. 日志页加载后自动启动监听。
2. 收到 `recognition-logs` 事件后列表实时追加。
3. 用户列表的刷新、添加、删除在 Tauri 构建下通过命令层工作。
4. 选择桌面图片后弹窗出现预览图。

## 8. 实施顺序

按依赖关系执行。这里有意不按 `T1 -> T2` 的任务编号顺序推进，而是先稳定命令合约，再接前端：

1. 修正 `websocket` 命令行为。
2. 在 `facedemo-rust/Cargo.toml` 和 `src-tauri/Cargo.toml` 中先声明 `tauri-backend` feature，并补齐相关依赖。
3. 重构 `backend.rs` 和 `app_controller.rs`，将 `fetch_logs()` 收窄为“启动监听”，并引入 `start_log_listener()`。
4. 完成日志页事件订阅和 `App` 中的自动启动接线。
5. 接入 `TauriInvokeBackend`，打通 Tauri 命令层。
6. 完成桌面图片选择命令与弹窗 UI 改造，同时保留 `dx serve` 的 Base64 降级路径。
7. 编写 `BUILD.md`。

## 9. 原则落地

### KISS

- 不增加快照协议。
- 不重写现有状态管理。
- 图片数据继续只保留 Base64 一个字段。

### YAGNI

- 不提前实现移动端拍照。
- 不为未来多来源日志做额外抽象。
- 不引入新的全局状态容器。

### DRY

- 后端命令映射统一收口在 `TauriInvokeBackend`。
- 日志写入统一走 `AppState::prepend_logs()`。

### SOLID

- `backend.rs` 负责后端访问抽象。
- `app_controller.rs` 负责状态编排。
- 页面组件只负责订阅和事件上抛。
- Tauri 命令只负责平台能力与桥接。
