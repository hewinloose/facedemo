# 项目功能流程文档

## 1. 项目概览

当前仓库的可运行业务代码集中在 `facedemo-rust/`，旧的 Ionic/Cordova 源码已经移除。现阶段项目由 3 个 Rust 子系统组成：

- `facedemo-rust/face-core`
  - 纯业务核心层。
  - 负责环境配置解析、百度人脸 API 请求构造、响应解析、WebSocket 日志消息解析。
- `facedemo-rust`
  - Dioxus 桌面前端壳。
  - 负责页面状态、页面交互、前端后端抽象、示例数据回退。
- `facedemo-rust/src-tauri`
  - Tauri v2 后端壳。
  - 负责命令暴露、HTTP 传输、WebSocket 长连接、向前端派发事件。

当前主功能仍围绕原系统的两条核心业务：

1. 人脸库管理
2. 识别日志查看

### 1.1 架构总览

```
┌──────────────────────────────────────────────────────┐
│               facedemo-rust（Dioxus UI）              │
│                                                      │
│  AppState ←── AppController ──► AppBackend (trait)   │
│                                  ↙           ↘       │
│              DemoBackend    ClientBackedBackend<T>    │
│              (示例数据)      (泛型，可替换传输层)      │
│                                   ↓                  │
│                     FaceClient<ReqwestTransport>      │
│                       ↓（直连，绕过 Tauri）           │
│                      百度人脸 API                     │
└──────────────────────────────────────────────────────┘
              ↕ invoke / event（可选链路）
┌──────────────────────────────────────────────────────┐
│              src-tauri（Tauri 命令层）                │
│                                                      │
│  Tauri Commands ──► AppServices ──► FaceClient<T>    │
│  WebSocket Listener ──────────────► emit(recognition-logs) │
└──────────────────────────────────────────────────────┘
                    ↓ 共享核心
┌──────────────────────────────────────────────────────┐
│               face-core（纯业务核心）                 │
│   BaiduFaceApi / FaceClient<T> / AppConfig           │
│   FaceApiTransport (trait) / WebSocket 解析           │
└──────────────────────────────────────────────────────┘
```

**关键架构特点**：前端 `facedemo-rust` 和后端 `src-tauri` 各自持有独立的 `ReqwestTransport` 实现，前者位于 `services/backend.rs`，后者位于 `src-tauri/src/runtime.rs`。当环境变量配置齐全时，前端 HTTP 模式**完全绕过 Tauri**，直连百度 API；只有需要调用原生能力（如相机）时才走 Tauri invoke 链路。

## 2. 目录职责

### 2.1 根目录

- `RUST_REFACTOR_PLAN.md`
  - Rust 重构方案说明。
- `FUNCTION_FLOW.md`
  - 当前文档。
- `.spec-workflow/`
  - 规格驱动开发流程资产，不参与运行时逻辑。
- `tmp_rust_check/`
  - Rust 工具链临时校验目录，不参与业务功能。

### 2.2 `facedemo-rust/`

- `src/main.rs`
  - Dioxus 前端启动入口。
- `src/app.rs`
  - 应用总装配。
  - 串起 `AppBackend`、`AppController`、`AppState` 和两个页面。
- `src/state/`
  - 前端状态模型和应用控制器。
- `src/services/backend.rs`
  - 前端后端抽象层。
  - 决定是走示例后端还是走真实 HTTP 后端。
- `src/pages/`
  - 两个主页面：人脸库、识别日志。
- `src/components/`
  - 添加用户弹窗、图片查看器。

### 2.3 `facedemo-rust/face-core/`

- `src/config.rs`
  - 统一解析 4 个关键环境变量。
- `src/face_api.rs`
  - 百度接口请求构造与响应解析。
- `src/client.rs`
  - 百度业务流程编排器。
- `src/websocket.rs`
  - 识别日志 WebSocket 消息解析。

### 2.4 `facedemo-rust/src-tauri/`

- `src/lib.rs`
  - Tauri 应用入口与命令注册。
- `src/commands/face_api.rs`
  - 人脸库相关 Tauri 命令。
- `src/commands/websocket.rs`
  - WebSocket 监听命令。
- `src/runtime.rs`
  - `reqwest` 传输实现与运行时常量。
- `src/state.rs`
  - Tauri 进程内共享服务容器。

## 3. 启动流程

### 3.0 快速启动

环境准备（`.env` 文件需放在 `facedemo-rust/` 目录下）：

```bash
# facedemo-rust/.env
BAIDU_CLIENT_ID=<your_client_id>
BAIDU_CLIENT_SECRET=<your_client_secret>
BAIDU_GROUP_ID=group1
WS_SERVER_URL=ws://<host>:<port>
```

启动命令：

```bash
# 方式一：启动 Dioxus 桌面前端（直连 HTTP 后端模式）
cd facedemo-rust
dx serve

# 方式二：启动完整 Tauri 桌面应用（含原生能力）
cd facedemo-rust
cargo tauri dev

# 运行所有测试
cargo test --workspace
```

不配置 `.env` 文件时，应用自动回退到 Demo 模式，无需任何网络连接即可运行。

### 3.1 Dioxus 前端启动流程

入口：`facedemo-rust/src/main.rs`

流程：

1. `main()` 调用 `dioxus::launch(App)`。
2. `App()` 在 `src/app.rs` 中创建 `AppBackend::from_env_or_demo()`。
3. `AppBackend` 读取环境变量：
   - `BAIDU_CLIENT_ID`
   - `BAIDU_CLIENT_SECRET`
   - `BAIDU_GROUP_ID`
   - `WS_SERVER_URL`
4. 如果 4 个变量齐全，则进入真实 HTTP 后端模式。
5. 如果缺任意变量，则回退到 `DemoBackend`。
6. `AppState` 以初始快照启动：
   - Demo 模式：预装示例用户、示例日志。
   - HTTP 模式：用户列表初始为空，日志先用示例种子，等待用户主动刷新。
7. 页面默认落在“人脸库”标签页。

### 3.2 Tauri 后端启动流程

入口：`facedemo-rust/src-tauri/src/main.rs`

流程：

1. `main()` 调用 `src_tauri::run()`。
2. `run()` 在 `src/lib.rs` 中加载环境配置。
3. 创建 `AppServices`：
   - `config`
   - `reqwest::Client`
   - `ReqwestTransport`
   - `websocket_task`
4. 将服务注入 Tauri 状态容器。
5. 注册命令：
   - `get_baidu_token`
   - `get_user_list`
   - `get_user_info`
   - `add_user`
   - `delete_user`
   - `start_websocket_listener`
6. Tauri 窗口配置来自 `src-tauri/tauri.conf.json`。

## 4. 核心配置流

统一配置模型位于 `face-core/src/config.rs` 的 `AppConfig`。

必须存在的环境变量：

- `BAIDU_CLIENT_ID`
- `BAIDU_CLIENT_SECRET`
- `BAIDU_GROUP_ID`
- `WS_SERVER_URL`

配置流转关系：

1. 前端 `HttpFaceBackend::try_from_env()` 读取环境变量。
2. Tauri `load_config_from_env()` 读取同一组环境变量。
3. 两端都通过 `AppConfig::from_map()` 做校验。
4. 缺任何字段都会返回 `ConfigError::MissingVar`。

这意味着当前前端直连 HTTP 模式和 Tauri 模式使用的是同一份业务配置约束。

## 5. 数据模型

以下是贯穿全系统的核心类型定义（均位于 `face-core/src/models/` 和 `face-core/src/face_api.rs`）：

```rust
// 用户摘要（人脸库条目）
struct FaceUserSummary {
    user_id:   String,   // 用户唯一标识
    user_info: String,   // 用户描述（如"前台"、"访客"）
}

// 识别日志条目
struct RecognitionLogEntry {
    result:    bool,     // 是否识别成功
    user_info: String,   // 识别到的用户描述（失败时为空字符串）
    date:      String,   // 时间戳字符串（如"2026-03-15 10:12:00"）
    image:     String,   // Base64 编码的人脸截图
}

// 添加用户入参（前端 → API）
struct NewFaceUser {
    user_id:      String,   // 用户 ID
    user_info:    String,   // 用户描述
    image_base64: String,   // Base64 编码的人脸图片
}

// 添加用户草稿（前端状态层，含校验逻辑）
struct UserDraft {
    user_id:      String,
    user_info:    String,
    image_base64: String,
    // can_submit()：三个字段均非空白时返回 true
    // as_new_user()：校验通过后转换为 NewFaceUser，否则返回 None
}
```

前端状态容器（`facedemo-rust/src/state/app_state.rs`）：

```rust
struct AppState {
    active_tab:           AppTab,         // 当前选中的标签页（默认 FaceLibrary）
    users:                Vec<FaceUserSummary>,
    logs:                 Vec<RecognitionLogEntry>,
    user_draft:           UserDraft,      // 添加用户弹窗的草稿状态
    show_add_user_modal:  bool,
    selected_log_image:   Option<String>, // 当前预览的日志图片 Base64
    status_message:       Option<String>, // 操作成功提示（绿色横幅）
    error_message:        Option<String>, // 错误提示（红色横幅）
}

enum AppTab {
    FaceLibrary,     // 默认值（#[default]）
    RecognitionLog,
}
```

## 6. 错误类型链路

项目使用分层错误模型，错误从底层向上转换：

```
ConfigError（face-core）
  ├── MissingVar(String)       ← 缺少必要环境变量
  └── ...

FaceApiError（face-core）
  ├── InvalidPayload(String)   ← JSON 反序列化失败
  ├── Remote(String)           ← 百度 API 返回错误（含 error_code）
  └── MissingField(&str)       ← 响应中缺少期望字段

TransportError（face-core）
  └── message: String          ← 网络请求失败

FaceClientError（face-core）
  ├── Transport(TransportError)
  └── Api(FaceApiError)

BackendError（facedemo-rust 前端层）
  └── message: String          ← 统一包装上游所有错误

UI 呈现：BackendError → AppState::set_error() → error_message → 红色横幅
```

**错误上浮路径示例**（刷新用户列表失败）：

```
reqwest 请求失败
  → TransportError
  → FaceClientError::Transport
  → BackendError（via From trait）
  → AppController::load_users() 返回 Err
  → AppState::set_error("...")
  → UI 渲染 error_message 红色横幅
```

## 7. 人脸库流程

### 7.1 页面侧流程

页面：`facedemo-rust/src/pages/face_library.rs`

页面暴露 3 个动作：

- 刷新
- 添加用户
- 删除用户

### 7.2 刷新用户流程

调用链：

1. 用户点击"刷新"。
2. 页面回调进入 `AppController::load_users()`。
3. 控制器调用 `FaceBackend::fetch_users()`。
4. 如果当前是 `DemoBackend`：
   - 直接返回内置示例用户。
5. 如果当前是 `HttpFaceBackend`：
   - 调用 `FaceClient::fetch_users(None)`。
6. `FaceClient` 内部流程：
   - 传入 `None` 触发 `resolve_token()` 重新获取 token
   - 调用百度分组用户列表接口
   - 得到 `user_id_list`
   - 逐个调用用户详情接口
   - 组装成 `Vec<FaceUserSummary>`
7. 控制器用 `state.replace_users()` 覆盖当前用户列表。
8. 状态栏更新为"已加载 N 个用户"。
9. 如果请求失败，则写入 `error_message`。

### 7.3 添加用户流程

调用链：

1. 用户点击"添加用户"。
2. `AppState::open_add_user_modal()` 打开弹窗（同时清空 `error_message`）。
3. 用户在 `UserInfoModal` 中填写：
   - `user_id`
   - `user_info`
   - `image_base64`（当前需手动粘贴 Base64 字符串）
4. 点击"提交"后，前端先调用 `UserDraft::as_new_user()` 做最小校验（三字段均不能为空白）。
5. 校验通过后进入 `AppController::add_user()`。
6. 控制器调用 `FaceBackend::add_user()`。
7. Demo 模式：
   - 直接把输入转换成 `FaceUserSummary` 返回。
8. HTTP 模式：
   - `FaceClient::add_user()` 固定传 `None` 重新获取 token
   - 调用百度"添加用户"接口（含 `image_type: BASE64`）
   - 校验成功响应
   - 再次调用用户详情接口
   - 返回新增后的标准用户摘要
9. 控制器调用 `state.upsert_user()`（已有则覆盖，不存在则追加到列表末尾）。
10. 成功后调用 `close_add_user_modal()`，内部自动执行 `user_draft.clear()`。
11. 状态栏更新为"已添加用户 xxx"。

### 7.4 删除用户流程

调用链：

1. 用户在列表中点击"删除"。
2. 页面将 `user_id` 传给 `AppController::delete_user()`。
3. 控制器调用 `FaceBackend::delete_user()`。
4. Demo 模式：
   - 直接返回成功。
5. HTTP 模式：
   - `FaceClient::delete_user()` 固定传 `None`，始终重新获取 token
   - 调用百度"删除用户"接口
   - 校验响应是否成功
6. 控制器调用 `state.remove_user(user_id)`。
7. 状态栏更新为"已删除用户 xxx"。

## 8. 识别日志流程

### 8.1 页面侧流程

页面：`facedemo-rust/src/pages/recognition_log.rs`

当前页面支持两个动作：

- 刷新日志
- 查看图片

### 8.2 刷新日志流程

调用链：

1. 用户点击“刷新”。
2. 页面进入 `AppController::refresh_logs()`。
3. 控制器调用 `FaceBackend::fetch_logs()`。
4. Demo 模式：
   - 返回内置示例日志。
5. HTTP 模式：
   - 当前仍返回 `logs_seed`，也就是示例日志。
   - 这里还没有接通真实 WebSocket 日志流。
6. 控制器调用 `state.prepend_logs(logs)`，把新日志插到最前面。
7. 状态栏更新为“已刷新 N 条日志”。

注意：

当前“刷新日志”是前端最明显的未闭环点。即使启用真实 HTTP 后端，日志仍然是示例数据，不是线上识别结果。

### 8.3 查看图片流程

调用链：

1. 用户在日志项中点击“查看图片”。
2. 页面把日志中的 `image` 传给 `AppState::select_log_image()`。
3. `ImageViewer` 组件将 Base64 拼接成 `data:image/png;base64,...`。
4. 弹出预览层。
5. 点击“关闭”后调用 `clear_selected_log_image()`。

## 9. 百度 API 业务流程

这部分集中在 `face-core`，是当前项目最稳定的共享核心。

### 9.1 请求构造

`face-core/src/face_api.rs` 中的 `BaiduFaceApi` 负责生成标准 `ApiRequest`：

- `token_request()`
- `user_list_request()`
- `user_detail_request()`
- `add_user_request()`
- `delete_user_request()`

统一特点：

- 路径、查询参数、JSON 请求体全部在这里收口。
- 所有请求都带 `group_id` 作用域。
- 添加用户使用 `image_type = BASE64`。

### 9.2 响应解析

`BaiduFaceApi` 同时负责解析响应：

- `parse_access_token()`
- `parse_user_ids()`
- `parse_user_detail()`
- `parse_success()`

解析策略：

- 先反序列化 JSON。
- 如果有 `error` 字段（字符串），按 OAuth 错误处理。
- 如果 `error_code` 存在且不为 `0`，按百度接口错误处理，同时附带 `error_msg`。
- 缺期望字段时返回 `FaceApiError::MissingField`。

### 9.3 流程编排

`face-core/src/client.rs` 中的 `FaceClient<T>` 负责流程级编排：

- `fetch_token()`
- `fetch_users()`
- `fetch_user_info()`
- `add_user()`
- `delete_user()`

核心意义：

- 把”取 token + 调用接口 + 解析结果”的重复逻辑抽到一处。
- Tauri 命令层和前端 HTTP 后端都复用同一套业务流程。
- Token 复用通过 `resolve_token(Option<&str>)` 实现：传 `None` 时重新获取，传入已有 token 则直接使用。

## 10. Tauri 命令流程

### 10.1 人脸库命令

文件：`facedemo-rust/src-tauri/src/commands/face_api.rs`

Tauri 命令本身很薄，只做三件事：

1. 从 `AppServices` 取配置和传输层。
2. 组装 `FaceClient<ReqwestTransport>`。
3. 调用共享核心方法并把错误转成字符串。

命令与核心方法映射：

- `get_baidu_token` -> `FaceClient::fetch_token`
- `get_user_list` -> `FaceClient::fetch_users`
- `get_user_info` -> `FaceClient::fetch_user_info`
- `add_user` -> `FaceClient::add_user`
- `delete_user` -> `FaceClient::delete_user`

### 10.2 WebSocket 命令

文件：`facedemo-rust/src-tauri/src/commands/websocket.rs`

流程：

1. 前端调用 `start_websocket_listener`。
2. Tauri 读取 `WS_SERVER_URL`。
3. 如果已经存在监听任务，则直接返回，避免重复连接。
4. 新建异步任务并连接 WebSocket。
5. 连接成功后先发送文本帧 `"s"` 作为订阅起始信号。
6. 持续读取消息：
   - 文本帧：交给 `face_core::websocket::parse_log_entries()`
   - 二进制帧：忽略
7. 解析成功后，向前端派发 `recognition-logs` 事件。

当前状态：

- Tauri 端的日志监听已具备。
- Dioxus 前端还没有正式订阅这个 Tauri 事件。
- 所以前端日志页还未真正消费实时识别流。

## 11. 前端后端切换策略

`src/services/backend.rs` 定义了当前前端的后端抽象。

### 11.1 `FaceBackend`

统一接口：

- `fetch_users`
- `add_user`
- `delete_user`
- `fetch_logs`

好处：

- 页面和控制器不依赖具体实现。
- Demo 模式和真实模式可以无缝切换。

### 11.2 `DemoBackend`

适用场景：

- 没有配置环境变量
- 本地只想验证 UI 和状态流
- 需要在无后端环境下保持页面可运行

### 11.3 `HttpFaceBackend`

`HttpFaceBackend` 是一个**类型别名**，而非独立结构体：

```rust
pub type HttpFaceBackend = ClientBackedBackend<ReqwestTransport>;
```

这意味着 `ClientBackedBackend<T>` 通过泛型参数 `T: FaceApiTransport` 支持任意传输实现，测试时可注入 mock transport，生产时使用 `ReqwestTransport`。

**双路径架构说明**：前端 `facedemo-rust` 和 Tauri 后端 `src-tauri` 各自持有一份独立的 `ReqwestTransport` 实现：

- `facedemo-rust/src/services/backend.rs`：前端直连模式使用
- `facedemo-rust/src-tauri/src/runtime.rs`：Tauri 命令模式使用

当环境变量齐全时，前端 HTTP 模式**完全绕过 Tauri**，直连百度 API。这是当前的"直连快捷路径"，在移动端需要原生能力后应迁移至 Tauri invoke 链路。

适用场景：

- 已配置百度相关环境变量
- 桌面端直连真实百度人脸接口（无需 Tauri 原生能力）

当前限制：

- 用户流是真实的。
- 日志流仍然是示例种子，不是真实 WebSocket 数据。

### 11.4 `AppBackend`

职责：

- 在应用启动时决定使用 Demo 还是 HTTP。
- 同时提供初始用户、初始日志、初始状态文案。

## 12. 当前已闭环与未闭环部分

### 12.1 已闭环

- 应用启动
- 模式切换：Demo / HTTP
- 用户列表刷新
- 用户添加
- 用户删除
- 日志列表展示（示例数据）
- 日志图片查看
- Tauri 端 WebSocket 监听组件与事件派发（`start_websocket_listener` 命令可用）
- 共享核心层的百度 API 编排

### 12.2 未闭环（含优先级）

| 优先级 | 未完成项 | 阻塞影响 |
|--------|---------|---------|
| P0 | Dioxus 前端订阅 Tauri `recognition-logs` 事件 | 实时识别日志完全不可用 |
| P0 | `fetch_logs()` 接通真实 WebSocket 数据流 | 同上，日志页永远展示示例数据 |
| P1 | 前端改为通过 Tauri `invoke` 调用后端命令 | 移动端无法使用直连 HTTP，必须走 Tauri |
| P2 | 相机/相册接入（替代手动粘贴 Base64） | 添加用户体验极差 |
| P3 | 完整打包链路文档（`dist/index.html` 说明） | 影响交付和部署流程 |

## 13. 测试覆盖说明

当前测试主要覆盖 3 层：

- `face-core/tests`
  - 配置加载
  - 百度请求构造
  - 业务流程编排
  - WebSocket 消息解析
- `facedemo-rust/tests`
  - 前端状态变更
  - 控制器行为
  - 后端抽象的真实/伪造调用
- `src-tauri/tests`
  - URL 构造
  - 事件名稳定性

这说明当前代码更偏“核心逻辑与状态流可验证”，而不是“桌面端集成联调已完成”。

## 14. 总结

当前项目已经完成从旧 Ionic 结构到 Rust 三层结构的最小业务迁移：

- `face-core` 负责可复用业务核心，是最稳定的部分
- `src-tauri` 负责桌面后端能力，WebSocket 监听已就绪但前端尚未接入
- `facedemo-rust` 前端已具备两条主流程的最小交互壳，支持 Demo / HTTP 双模式

**下一步**：按 Section 12.2 的优先级顺序，优先打通 Tauri 事件订阅（P0），再推进 Tauri invoke 收口（P1），最后接入原生相机（P2）。
