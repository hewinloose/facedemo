# 代码改造任务清单

> 来源：对 `FUNCTION_FLOW.md` 的评估分析
> 每个任务包含：目标文件、具体改动、验收标准

---

## 任务总览

| ID | 优先级 | 任务 | 涉及文件数 |
|----|--------|------|-----------|
| T1 | P0 | 前端订阅 `recognition-logs` 事件 + tab 切换自动启动监听 | 3 |
| T2 | P0 | 修复 WebSocket 重连缺陷 + `fetch_logs()` 职责收窄 | 2 |
| T3 | P1 | 用 Cargo feature 门控 Tauri invoke 后端 | 4 |
| T4 | P2 | 桌面文件选择接入（移动端相机预留接口） | 4 |
| T5 | P3 | 补充打包链路说明文档 | 1（文档） |

---

## T1 · 前端订阅 Tauri `recognition-logs` 事件

**优先级**：P0
**阻塞影响**：实时识别日志功能完全不可用

### 背景

`src-tauri/src/commands/websocket.rs` 已实现 WebSocket 监听并通过
`app_handle.emit(RECOGNITION_LOG_EVENT, logs)` 向前端派发事件，
事件名常量 `RECOGNITION_LOG_EVENT = "recognition-logs"` 定义于
`src-tauri/src/runtime.rs:7`。

`facedemo-rust/src/pages/recognition_log.rs` 目前只有一个手动"刷新"按钮，
没有任何事件订阅代码；`app.rs` 在切换到日志 tab 时也没有触发监听的逻辑。

### 需要改动的文件

#### 文件 1：`facedemo-rust/src/app.rs`

**改动一：tab 切换时自动启动监听**

在 `AppTab::RecognitionLog` 分支渲染前，用 `use_effect` 检测到切换到日志页时
触发一次 `start_websocket_listener`。这里是发起 invoke 的正确位置，
因为页面组件本身没有直接调用 invoke 的能力：

```rust
// app.rs - AppTab::RecognitionLog 分支
let ws_controller = controller.clone();
use_effect(move || {
    // 仅在 tauri-backend feature 下编译此段
    #[cfg(feature = "tauri-backend")]
    {
        let ws_controller = ws_controller.clone();
        let mut state = state;
        spawn(async move {
            let mut next = state.read().clone();
            ws_controller.start_log_listener(&mut next).await;
            state.set(next);
        });
    }
});
```

`use_effect` 依赖项为空（`[]`），仅在组件挂载时执行一次。
切换回人脸库再切回来时，若连接仍在运行，Tauri 命令内部幂等处理，不会重复连接。
若连接已断，命令会检测到旧句柄已结束并重建（见 T2 对命令本体的修改）。

**改动二：新增 `on_new_logs` 回调传给页面**

将收到的实时日志通过 `state.prepend_logs()` 写入状态：

```rust
on_new_logs: move |logs: Vec<RecognitionLogEntry>| {
    let mut next = state.read().clone();
    next.prepend_logs(logs);
    state.set(next);
},
```

#### 文件 2：`facedemo-rust/src/state/app_controller.rs`

新增 `start_log_listener()` 方法，封装"触发监听 + 更新状态栏"逻辑：

```rust
pub async fn start_log_listener(&self, state: &mut AppState) {
    match self.backend.fetch_logs().await {
        Ok(_) => state.set_status("日志监听已启动"),
        Err(e) => state.set_error(format!("启动监听失败：{e}")),
    }
}
```

#### 文件 3：`facedemo-rust/src/pages/recognition_log.rs`

新增 `on_new_logs` prop 和事件订阅，将 Tauri 事件接入页面：

```rust
#[component]
pub fn RecognitionLogPage(
    logs: Vec<RecognitionLogEntry>,
    on_refresh: EventHandler<MouseEvent>,   // 保留：作为"重试/重新连接"入口
    on_select_image: EventHandler<String>,
    on_new_logs: EventHandler<Vec<RecognitionLogEntry>>,  // 新增
) -> Element {
    // 订阅 Tauri 事件，收到数据时通知父级
    #[cfg(feature = "tauri-backend")]
    use_effect(move || {
        listen("recognition-logs", move |logs| {
            on_new_logs.call(logs);
        });
    });
    // ... 原有 rsx! 不变
}
```

**"刷新"按钮语义变更**：文案从"刷新"改为"重新连接"，`on_refresh` 回调
触发 `start_log_listener()`，语义为"主动重试连接"而非"拉取新数据"。

### 验收

- 切换到识别日志页，状态栏出现"日志监听已启动"，无需手动点击。
- WebSocket 推送数据时，日志列表自动追加，无需点击任何按钮。
- 网络断开恢复后，点击"重新连接"，连接重建，日志继续追加。
- `dx serve` 下（无 `tauri-backend` feature），`use_effect` 中的 invoke
  代码不参与编译，页面正常渲染，不会 panic。

### 依赖

- T2 中对 `start_websocket_listener` 的命令修改须同步完成，否则"重新连接"
  在连接断开后仍无法重建。
- 需要在 `facedemo-rust/Cargo.toml` 中添加 Tauri 事件监听绑定依赖
  （`tauri-sys` 或 Dioxus 对应版本的 Tauri 集成包）。

---

## T2 · 修复 WebSocket 重连缺陷 + 接通真实日志数据

**优先级**：P0
**阻塞影响**：连接断开后无法重建；日志页面永远展示示例种子数据

### 背景

**缺陷一：`start_websocket_listener` 无法重连**

`src-tauri/src/state.rs:14` 中任务槽类型为 `Option<JoinHandle<()>>`。
`JoinHandle` 在任务退出（正常结束或 panic）后，`is_some()` 仍然为 `true`，
但 `handle.is_finished()` 会返回 `true`。

当前命令只检查 `is_some()`：

```rust
// websocket.rs:16-18 — 当前代码（有缺陷）
if guard.is_some() {
    return Ok(());   // ← 连接已断但句柄未清，永远走这里
}
```

WebSocket 服务端重启或网络中断后，任务退出，但槽位仍被占据，
再次调用命令不会重建连接。

**缺陷二：`fetch_logs()` 返回固定种子数据**

`facedemo-rust/src/services/backend.rs:157-159` 的 `ClientBackedBackend::fetch_logs()`
始终返回 `self.logs_seed.clone()`，与真实 WebSocket 数据无关。

### 需要改动的文件

#### 文件 1：`facedemo-rust/src-tauri/src/commands/websocket.rs`

修复幂等判断，区分"运行中"和"已结束"两种状态：

```rust
#[tauri::command]
pub async fn start_websocket_listener(
    app: AppHandle,
    services: State<'_, AppServices>,
) -> Result<(), String> {
    let ws_url = services.config.ws_url.clone();
    let task_slot = services.websocket_task.clone();
    let mut guard = task_slot.lock().await;

    // 修改点：is_some() 改为检查任务是否仍在运行
    if let Some(handle) = guard.as_ref() {
        if !handle.is_finished() {
            return Ok(());   // 真正运行中，幂等跳过
        }
        // 任务已退出（连接断开），清掉旧句柄，继续往下重建
    }
    *guard = None;

    // 以下建立连接逻辑与原来相同 ...
}
```

**改动范围**：仅命令函数开头的幂等判断部分，连接建立和消息处理逻辑不变。

#### 文件 2：`facedemo-rust/src/services/backend.rs`

`ClientBackedBackend::fetch_logs()` 职责收窄为"触发监听启动"，
不再返回日志列表（实时数据由 T1 的事件订阅接收）：

```rust
// 改造后：返回类型从 Result<Vec<...>> 改为 Result<(), ...>
async fn fetch_logs(&self) -> Result<(), BackendError> {
    #[cfg(feature = "tauri-backend")]
    {
        invoke("start_websocket_listener", ())
            .await
            .map_err(|e| BackendError::new(e.to_string()))
    }
    #[cfg(not(feature = "tauri-backend"))]
    {
        Ok(())   // dx serve 下无操作，日志由 Demo 种子或 HTTP 模式提供
    }
}
```

同步修改 `FaceBackend` trait 中 `fetch_logs` 的签名：

```rust
// services/backend.rs - FaceBackend trait
async fn fetch_logs(&self) -> Result<(), BackendError>;
```

`DemoBackend::fetch_logs()` 对应改为返回 `Ok(())`。

### 验收

| 场景 | 预期结果 |
|------|---------|
| 首次启动，WebSocket 服务在线 | 连接建立，日志开始流入 |
| 连接正常时切走再切回日志页 | 无重复连接，数据继续流入 |
| WebSocket 服务重启，点"重新连接" | 旧句柄被清除，新连接建立 |
| WebSocket 服务重启，再次切换到日志页 | 自动检测断连，重建连接 |
| `dx serve` 下点"重新连接" | 无操作，无 panic，状态不变 |

### 依赖

- 文件 2 中 `fetch_logs` 签名变更会影响 `AppController::refresh_logs()`
  的调用处，须同步检查 `app_controller.rs` 中的处理逻辑。

---

## T3 · 用 Cargo feature 门控 Tauri invoke 后端

**优先级**：P1
**阻塞影响**：移动端必须走 Tauri 命令层，当前代码无法支持

### 背景

`dx serve` 启动的是纯 Dioxus 桌面二进制（`facedemo-rust/Cargo.toml` 中
无 `tauri` 依赖），进程内不存在 Tauri IPC，调用 `invoke()` 会直接失败。
因此**不能用运行时检测**来决定走哪条路，必须在编译期分离。

三条链路的本质差异：

| 启动方式 | Tauri 运行时 | 应使用的后端 |
|----------|-------------|-------------|
| `dx serve` | **无** | `HttpFaceBackend` 或 `DemoBackend` |
| `cargo tauri dev` | **有** | `TauriInvokeBackend` |
| Android/iOS 打包 | **有** | `TauriInvokeBackend` |

### 需要改动的文件

#### 文件 1：`facedemo-rust/Cargo.toml`

新增一个 Cargo feature，由 `src-tauri` 在引用前端 crate 时激活：

```toml
[features]
default = []
tauri-backend = []
```

#### 文件 2：`facedemo-rust/src/services/backend.rs`

在 `#[cfg(feature = "tauri-backend")]` 门控下新增 `TauriInvokeBackend`，
实现 `FaceBackend` trait，每个方法通过 `invoke()` 调用对应 Tauri 命令：

| 方法 | Tauri 命令 |
|------|-----------|
| `fetch_users()` | `get_user_list` |
| `add_user(new_user)` | `add_user` |
| `delete_user(user_id)` | `delete_user` |
| `fetch_logs()` | `start_websocket_listener` |

```rust
#[cfg(feature = "tauri-backend")]
pub struct TauriInvokeBackend;

#[cfg(feature = "tauri-backend")]
#[async_trait]
impl FaceBackend for TauriInvokeBackend {
    async fn fetch_users(&self) -> Result<Vec<FaceUserSummary>, BackendError> {
        invoke("get_user_list", ())
            .await
            .map_err(|e| BackendError::new(e.to_string()))
    }
    // ... 其余方法类似
}
```

`AppBackend` 枚举同样用 feature 门控增加 variant：

```rust
pub enum AppBackend {
    Demo(DemoBackend),
    Http(HttpFaceBackend),
    #[cfg(feature = "tauri-backend")]
    Tauri(TauriInvokeBackend),
}
```

#### 文件 3：`facedemo-rust/src/app.rs`

`from_env_or_demo()` 用 `cfg` 属性在编译期选择入口，无任何运行时判断：

```rust
pub fn from_env_or_demo() -> Self {
    #[cfg(feature = "tauri-backend")]
    return Self::Tauri(TauriInvokeBackend);

    #[cfg(not(feature = "tauri-backend"))]
    {
        HttpFaceBackend::try_from_env()
            .map(Self::Http)
            .unwrap_or_else(|_| Self::Demo(DemoBackend))
    }
}
```

#### 文件 4：`facedemo-rust/src-tauri/Cargo.toml`

引用前端 crate 时激活 `tauri-backend` feature，使 Tauri 二进制自动编入
`TauriInvokeBackend`，而 `dx serve` 编译时不触碰这个依赖：

```toml
[dependencies]
facedemo-rust-lib = { path = "..", features = ["tauri-backend"] }
```

**验收**：
- `dx serve` 编译不含 `tauri-backend`，走 `HttpFaceBackend` / `DemoBackend`，行为与改动前完全一致。
- `cargo tauri dev` 和 Android 打包含 `tauri-backend`，所有 API 调用走 `TauriInvokeBackend`。
- 两个二进制互不影响，无运行时分支判断，无 panic 风险。

### 依赖

- 无强依赖，可与 T1/T2 并行开发。
- 建议 T1 完成后再合并 T3，避免 `fetch_logs` 相关改动产生冲突。

---

## T4 · 图片选取接入（替代手动粘贴 Base64）

**优先级**：P2
**阻塞影响**：添加用户时需要手动粘贴 Base64 字符串，体验极差

### 本次实施范围

**本次做**：桌面文件选择对话框 + Tauri 端统一返回 Base64
**预留接口**：移动端真机拍照（命令签名固定，内部实现后补）

> 移动端相机依赖 T3 完成后才能端到端验收，且需要真机和额外权限配置，
> 不在本次范围内。命令接口设计时预留移动端扩展点，前端代码届时无需改动。

### 背景

`facedemo-rust/src/components/user_info_modal.rs` 目前只有文本输入框
接收 `image_base64`。本次将其替换为文件选择按钮，由 Tauri 命令层统一
处理"打开对话框 → 读取图片 → Base64 编码 → 返回字符串"全流程。

### 需要改动的文件

#### 文件 1：`facedemo-rust/src/components/user_info_modal.rs`

将"粘贴 Base64"文本输入框替换为"选择图片"按钮，点击后通过
`invoke("pick_image", ())` 调用 Tauri 命令，获取到 Base64 字符串后
写入 `image_base64` 状态并显示缩略图预览：

```rust
button {
    onclick: move |_| {
        spawn(async move {
            match invoke::<String>("pick_image", ()).await {
                Ok(base64) => on_image_input.call(base64),
                Err(e) => on_error.call(e.to_string()),
            }
        });
    },
    "选择图片"
}
// 图片预览（base64 非空时显示）
if !image_base64.is_empty() {
    img { src: "data:image/jpeg;base64,{image_base64}", class: "preview" }
}
```

**注意**：前端只调用 `pick_image`，不区分桌面/移动端，该差异完全封装
在 Tauri 命令内部。

#### 文件 2：`facedemo-rust/src-tauri/src/commands/camera.rs`（新建）

实现 `pick_image` 命令，当前实现桌面文件对话框，移动端以
`#[cfg(target_os)]` 预留桩：

```rust
#[tauri::command]
pub async fn pick_image() -> Result<String, String> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        // 桌面端：rfd 文件对话框
        let path = rfd::FileDialog::new()
            .add_filter("图片", &["jpg", "jpeg", "png", "bmp"])
            .pick_file()
            .ok_or_else(|| "未选择文件".to_string())?;
        let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
        Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        // 移动端：待 T3 完成后接入 tauri-plugin-camera
        Err("移动端相机接口尚未实现，请在 T3 完成后补充".to_string())
    }
}
```

#### 文件 3：`facedemo-rust/src-tauri/src/commands/mod.rs`

导出新模块：

```rust
pub mod camera;
pub mod face_api;
pub mod websocket;
```

#### 文件 4：`facedemo-rust/src-tauri/src/lib.rs`

在 `invoke_handler` 中注册新命令：

```rust
.invoke_handler(tauri::generate_handler![
    commands::face_api::get_baidu_token,
    commands::face_api::get_user_list,
    commands::face_api::get_user_info,
    commands::face_api::add_user,
    commands::face_api::delete_user,
    commands::websocket::start_websocket_listener,
    commands::camera::pick_image,   // 新增
])
```

### 新增依赖（`src-tauri/Cargo.toml`）

```toml
rfd     = "0.15"   # 桌面端文件选择对话框（仅桌面端编译）
base64  = "0.22"   # Base64 编码
```

### 验收标准

| 场景 | 预期结果 |
|------|---------|
| `cargo tauri dev` 桌面端点击"选择图片" | 弹出系统文件对话框，选择后出现图片预览 |
| 取消选择 | 提示"未选择文件"，`image_base64` 不变 |
| 选择非图片文件 | 对话框 filter 已限制，不可选中 |
| 填写完整后提交 | 图片正常上传百度人脸 API，用户出现在列表中 |
| Android/iOS 调用 | 返回错误提示，不崩溃（预留桩正常工作） |

### 移动端后续任务（T4-mobile，T3 完成后执行）

T3 完成、真机环境就绪后，`camera.rs` 中 `#[cfg(android/ios)]` 分支替换为：

```toml
# src-tauri/Cargo.toml 届时追加
tauri-plugin-camera = "2"
```

```rust
// camera.rs 移动端分支替换为
#[cfg(any(target_os = "android", target_os = "ios"))]
{
    use tauri_plugin_camera::CameraExt;
    let photo = app.camera().take_photo(Default::default())
        .map_err(|e| e.to_string())?;
    Ok(photo.base64_data)
}
```

Android 还需在 `gen/android/app/src/main/AndroidManifest.xml` 补充：

```xml
<uses-permission android:name="android.permission.CAMERA" />
```

---

## T5 · 补充打包链路文档

**优先级**：P3
**阻塞影响**：交付和部署流程不透明，新成员无法独立打包

### 背景

`dist/index.html` 目前只是最小前端分发壳，没有说明如何从源码到最终 APK。

### 需要改动的文件

#### 文件（新建）：`facedemo-rust/BUILD.md`

需要覆盖以下内容：

1. **前置依赖**
   - Rust 工具链版本要求
   - Android NDK / SDK 版本
   - `dx`（Dioxus CLI）安装命令
   - Tauri CLI 安装命令

2. **桌面端构建**
   ```bash
   cd facedemo-rust
   cargo tauri build
   # 产物路径：target/release/bundle/
   ```

3. **Android 端构建**
   ```bash
   cargo tauri android build
   # 产物路径：gen/android/app/build/outputs/apk/
   ```

4. **签名配置**
   - keystore 文件位置
   - `tauri.conf.json` 中签名相关字段说明
   - 如何通过环境变量传入 keystore 密码（避免明文写入配置）

5. **`.env` 文件说明**
   - 必填变量列表及含义（与 Section 3.0 保持一致）
   - Demo 模式下不需要 `.env` 的说明

**验收**：新成员按 `BUILD.md` 操作，能在干净环境中成功打出可安装的 APK。

---

## 执行顺序建议

```
T1（订阅事件）
  └─► T2（接通日志流）
        └─► T3（invoke 收口）
              └─► T4（相机接入）
                    └─► T5（打包文档）
```

T1 → T2 有强依赖关系，必须串行。
T3 可与 T1/T2 并行开发，但建议 T1 稳定后再合并，避免冲突。
T4、T5 相互独立，可随时并行。
