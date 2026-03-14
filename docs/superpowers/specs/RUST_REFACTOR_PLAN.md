# 人脸识别预警系统 — Rust 重构方案

> 原项目：Ionic 5 + Angular 9 + Cordova（移动端混合应用）
> 目标技术栈：Rust + Tauri v2 + Dioxus（跨平台原生移动应用）
> 文档日期：2026-03-15

---

## 一、现状分析

### 当前技术栈

| 层次 | 当前实现 | 问题点 |
|------|----------|--------|
| UI 框架 | Angular 9 + Ionic 5 | 版本老旧（2020年），TypeScript 构建链复杂 |
| 移动容器 | Cordova | 已被弃用，WebView 性能差 |
| HTTP 通信 | Axios + Cordova Native HTTP | 双套 HTTP 层，维护成本高 |
| WebSocket | 浏览器原生 WebSocket | 依赖 WebView 环境 |
| 构建工具 | Angular CLI + Gradle | 依赖 Node.js 环境，包体积大（57MB）|
| 安全 | API 密钥硬编码在源码中 | 严重安全隐患 |

### 核心功能清单

```
人脸库管理（Tab1）
  ├── 获取百度 OAuth2 Token
  ├── 获取/展示人脸用户列表
  ├── 添加用户（拍照 / 选择相册 → Base64 → 上传百度 API）
  └── 删除用户

识别日志（Tab2）
  ├── WebSocket 实时接收识别结果（ws://47.113.92.62:8081）
  ├── 日志列表展示（成功/失败）
  └── 点击查看识别人脸图片
```

---

## 二、Rust 技术选型

### 2.1 框架比较

| 框架 | 定位 | Android/iOS | 生态成熟度 | 推荐度 |
|------|------|-------------|-----------|--------|
| **Tauri v2** | 应用容器 + Rust 后端 | ✅ 官方支持 | ⭐⭐⭐⭐⭐ | ★★★★★ |
| **Dioxus** | Rust 原生 UI | ✅ 移动端支持 | ⭐⭐⭐⭐ | ★★★★☆ |
| **Slint** | 声明式 Rust UI | ✅ 商用可用 | ⭐⭐⭐⭐ | ★★★☆☆ |
| **egui** | 即时模式 GUI | ⚠️ 有限支持 | ⭐⭐⭐ | ★★☆☆☆ |

### 2.2 推荐方案：Tauri v2 + Dioxus

```
┌─────────────────────────────────────┐
│            Tauri v2 容器            │  ← 负责打包、权限、原生 API
│  ┌───────────────────────────────┐  │
│  │       Dioxus UI (Rust)        │  │  ← 响应式 UI 框架
│  │   (替代 Angular + Ionic)      │  │
│  └───────────────────────────────┘  │
│  ┌───────────────────────────────┐  │
│  │      Rust 业务逻辑层          │  │  ← 替代 Angular Services
│  │  reqwest / tokio-tungstenite  │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
```

**选型理由：**
- Tauri v2 正式支持 Android 和 iOS，是目前 Rust 移动端最成熟方案
- Dioxus 语法类似 React/Angular，迁移学习曲线最平缓
- 整个栈均为 Rust，无 JavaScript 运行时，性能和安全性大幅提升
- 包体积可从 57MB 压缩至 **约 5-8MB**

---

## 三、目标架构设计

### 3.1 目录结构

```
facedemo-rust/
├── Cargo.toml                    # 工作区配置
├── Cargo.lock
├── tauri.conf.json               # Tauri 应用配置
├── .env                          # 环境变量（替代硬编码密钥）
│
├── src-tauri/                    # Rust 后端（Tauri 命令）
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs               # Tauri 入口
│   │   ├── lib.rs
│   │   ├── commands/             # Tauri 命令（替代 Angular Services）
│   │   │   ├── mod.rs
│   │   │   ├── face_api.rs       # 百度人脸 API 封装
│   │   │   └── websocket.rs      # WebSocket 连接管理
│   │   ├── models/               # 数据模型
│   │   │   ├── mod.rs
│   │   │   ├── user.rs           # 用户/人脸模型
│   │   │   └── log_entry.rs      # 识别日志模型
│   │   └── config.rs             # 配置管理（读取 .env）
│   └── capabilities/             # Tauri 权限配置
│       └── mobile.json
│
├── src/                          # Dioxus 前端（Rust UI）
│   ├── main.rs                   # 前端入口
│   ├── app.rs                    # 根组件（替代 AppComponent）
│   ├── pages/                    # 页面（替代 tab1, tab2）
│   │   ├── mod.rs
│   │   ├── face_library.rs       # 人脸库页面（Tab1）
│   │   └── recognition_log.rs    # 识别日志页面（Tab2）
│   ├── components/               # 可复用组件
│   │   ├── mod.rs
│   │   ├── user_info_modal.rs    # 添加用户弹窗
│   │   └── image_viewer.rs       # 图片查看弹窗
│   ├── state/                    # 全局状态（替代 Angular 服务）
│   │   ├── mod.rs
│   │   └── app_state.rs
│   └── theme/                    # 样式主题
│       └── variables.rs
│
├── gen/                          # Tauri 生成的原生项目
│   ├── android/
│   └── apple/
│
└── tests/                        # 集成测试
    ├── face_api_tests.rs
    └── websocket_tests.rs
```

### 3.2 数据流架构

```
用户操作（Dioxus UI）
       │
       ▼
  invoke!() 宏调用 Tauri 命令
       │
       ▼
  Rust 后端命令（src-tauri/commands/）
       │
       ├──► reqwest → 百度人脸 API → 返回结果
       │
       └──► tokio-tungstenite → WebSocket 服务器
                                      │
                                      ▼
                               emit!() 事件推送
                                      │
                                      ▼
                            Dioxus 状态更新 → UI 重渲染
```

---

## 四、核心依赖清单

### 4.1 Cargo.toml（src-tauri）

```toml
[dependencies]
# Tauri 核心
tauri = { version = "2.x", features = ["protocol-asset"] }
tauri-plugin-camera = "2.x"          # 替代 @ionic-native/camera
tauri-plugin-http = "2.x"            # 替代 @ionic-native/http

# 异步运行时
tokio = { version = "1", features = ["full"] }

# HTTP 客户端（替代 axios）
reqwest = { version = "0.12", features = ["json", "multipart"] }

# WebSocket 客户端（替代浏览器 WebSocket）
tokio-tungstenite = { version = "0.24", features = ["native-tls"] }

# 序列化（替代 JSON.parse / JSON.stringify）
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Base64 编码（替代 @ionic-native/base64）
base64 = "0.22"

# 配置和环境变量（替代硬编码密钥）
dotenvy = "0.15"

# 错误处理
anyhow = "1"
thiserror = "2"

# 日志
tracing = "0.1"
tracing-subscriber = "0.3"
```

### 4.2 Cargo.toml（src/ — Dioxus 前端）

```toml
[dependencies]
# Dioxus UI 框架（替代 Angular + Ionic）
dioxus = { version = "0.6", features = ["mobile"] }

# Tauri 前端绑定
wry = "0.46"

# 图片处理
image = "0.25"
```

---

## 五、模块迁移对照表

### 5.1 Angular → Dioxus 组件映射

| Angular/Ionic | Rust/Dioxus | 说明 |
|--------------|-------------|------|
| `AppComponent` | `app.rs::App` | 根组件 |
| `TabsPage` | `app.rs` 中的 Tab 路由 | 底部导航 |
| `Tab1Page` (人脸库) | `pages/face_library.rs::FaceLibraryPage` | 人脸库主页 |
| `Tab2Page` (日志) | `pages/recognition_log.rs::RecognitionLogPage` | 识别日志页 |
| `UserInfoComponent` | `components/user_info_modal.rs` | 添加用户弹窗 |
| `ImageComponent` | `components/image_viewer.rs` | 图片查看弹窗 |
| `HttpserviceService` | `commands/face_api.rs` | Tauri Rust 命令 |

### 5.2 TypeScript 业务逻辑 → Rust 实现

**获取 Token（tab1.page.ts → face_api.rs）**

```typescript
// 旧：TypeScript
async getToken(): Promise<string> {
  const res = await axios.post(`https://aip.baidubce.com/oauth/2.0/token`, {
    grant_type: 'client_credentials',
    client_id: 'rTD5KIe8AnN7rUy7O1nCpqto',  // ← 硬编码风险
    client_secret: 'ZfrkZKt5fTqLh9e17OknYQk7QxuuBEpR'
  });
  return res.data.access_token;
}
```

```rust
// 新：Rust（安全，从 .env 读取）
#[tauri::command]
pub async fn get_baidu_token(config: tauri::State<'_, AppConfig>) -> Result<String, String> {
    let client = reqwest::Client::new();
    let resp: serde_json::Value = client
        .post("https://aip.baidubce.com/oauth/2.0/token")
        .query(&[
            ("grant_type", "client_credentials"),
            ("client_id", &config.client_id),       // ← 从 .env 读取
            ("client_secret", &config.client_secret),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    resp["access_token"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or("Token 字段不存在".to_string())
}
```

**WebSocket 实时推送（tab2.page.ts → websocket.rs）**

```typescript
// 旧：TypeScript（浏览器 WebSocket）
const ws = new WebSocket('ws://47.113.92.62:8081');
ws.onmessage = (event) => {
  this.logs = JSON.parse(event.data);
};
```

```rust
// 新：Rust（tokio-tungstenite，服务器端持久连接）
#[tauri::command]
pub async fn start_websocket_listener(app_handle: tauri::AppHandle) {
    tokio::spawn(async move {
        let (ws_stream, _) = tokio_tungstenite::connect_async(
            "ws://47.113.92.62:8081"
        ).await.expect("WebSocket 连接失败");

        let (mut write, mut read) = ws_stream.split();
        write.send(Message::Text("s".to_string())).await.ok();

        while let Some(Ok(msg)) = read.next().await {
            if let Message::Text(data) = msg {
                if let Ok(logs) = serde_json::from_str::<Vec<LogEntry>>(&data) {
                    app_handle.emit("recognition-logs", logs).ok();
                }
            }
        }
    });
}
```

### 5.3 安全改进：密钥管理

```bash
# .env 文件（不提交 Git）
BAIDU_CLIENT_ID=rTD5KIe8AnN7rUy7O1nCpqto
BAIDU_CLIENT_SECRET=ZfrkZKt5fTqLh9e17OknYQk7QxuuBEpR
BAIDU_GROUP_ID=group1
WS_SERVER_URL=ws://47.113.92.62:8081
```

```rust
// config.rs：启动时读取配置
#[derive(Clone)]
pub struct AppConfig {
    pub client_id: String,
    pub client_secret: String,
    pub group_id: String,
    pub ws_url: String,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        Ok(AppConfig {
            client_id: std::env::var("BAIDU_CLIENT_ID")?,
            client_secret: std::env::var("BAIDU_CLIENT_SECRET")?,
            group_id: std::env::var("BAIDU_GROUP_ID")?,
            ws_url: std::env::var("WS_SERVER_URL")?,
        })
    }
}
```

---

## 六、迁移实施步骤

### 阶段一：环境搭建（预计 1 天）

- [ ] 安装 Rust 工具链（`rustup`）
- [ ] 安装 Tauri CLI v2：`cargo install tauri-cli --version "^2"`
- [ ] 安装 Android NDK 和 SDK（Tauri 移动端依赖）
- [ ] 创建新项目：`cargo tauri init`
- [ ] 配置 Android 目标：`cargo tauri android init`
- [ ] 验证 Hello World 可在 Android 模拟器运行

### 阶段二：后端核心逻辑（预计 3 天）

- [ ] 实现 `config.rs`：环境变量加载
- [ ] 实现 `models/user.rs` 和 `models/log_entry.rs`：数据结构
- [ ] 实现 `commands/face_api.rs`：
  - [ ] `get_baidu_token`：获取 OAuth Token
  - [ ] `get_user_list`：获取用户列表
  - [ ] `get_user_info`：获取用户详情
  - [ ] `add_user`：添加人脸用户（含 Base64 图片）
  - [ ] `delete_user`：删除用户
- [ ] 实现 `commands/websocket.rs`：WebSocket 实时监听
- [ ] 编写单元测试（各 API 命令）

### 阶段三：前端 UI 开发（预计 4 天）

- [ ] 搭建 Dioxus 项目结构
- [ ] 实现底部 Tab 导航（替代 Ionic Tabs）
- [ ] 实现 `pages/face_library.rs`：
  - [ ] 用户列表展示（替代 Tab1）
  - [ ] 下拉刷新
  - [ ] 添加/删除用户操作
- [ ] 实现 `pages/recognition_log.rs`：
  - [ ] 日志列表实时更新（替代 Tab2）
  - [ ] 点击查看图片
- [ ] 实现 `components/user_info_modal.rs`：
  - [ ] 拍照 / 相册选择（调用 Tauri Camera 插件）
  - [ ] 图片预览
  - [ ] 提交上传
- [ ] 实现基本主题样式（模仿 Ionic 风格）

### 阶段四：集成测试与调优（预计 2 天）

- [ ] 联调百度 API（Token → 用户列表 → 添加/删除）
- [ ] 联调 WebSocket 实时日志
- [ ] Android 真机测试
- [ ] 性能对比（包体积、启动时间、内存占用）
- [ ] 修复兼容性问题

### 阶段五：发布打包（预计 1 天）

- [ ] 配置 Android 签名（`.env` 管理 keystore 密码）
- [ ] 构建 Release APK：`cargo tauri android build`
- [ ] 验证安装和功能完整性

---

## 七、预期收益对比

| 指标 | 当前（Ionic/Cordova） | 重构后（Rust/Tauri） | 改善 |
|------|----------------------|---------------------|------|
| APK 大小 | ~13MB (debug) / 11MB (release) | ~5-7MB (release) | **↓ 50%** |
| 冷启动时间 | ~2-3 秒 | ~0.5-1 秒 | **↓ 70%** |
| 内存占用 | ~80-120MB | ~20-40MB | **↓ 70%** |
| JS 运行时 | 需要（WebView） | 无 | **消除** |
| 安全性 | API 密钥硬编码 | 环境变量隔离 | **显著提升** |
| 类型安全 | TypeScript（部分） | Rust（全量编译检查） | **显著提升** |
| 框架维护 | Cordova（已弃用） | Tauri v2（活跃维护） | **长期可持续** |

---

## 八、风险与缓解措施

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| Dioxus 移动端生态尚不成熟 | 部分 Ionic 组件无对应实现 | 自行实现基础组件；或考虑 Tauri + WebView 混合方案 |
| Tauri v2 Android 打包配置复杂 | 构建失败 | 参考官方示例项目；预留环境搭建时间 |
| 百度 API CORS 限制 | WebView 请求被拦截 | 使用 Tauri 后端命令发起请求（绕过 CORS） |
| 相机/相册插件兼容性 | 原生功能失效 | 优先使用 `tauri-plugin-camera`；降级使用文件选择器 |
| Rust 学习曲线 | 开发效率低 | 重点掌握 ownership、async/await、serde 三个核心概念 |

---

## 九、备选方案

### 方案 B：Tauri v2 + 保留 Angular 前端

若 Rust UI 开发成本过高，可采用折中方案：
- **后端**：迁移到 Rust（Tauri Commands 处理 API 和 WebSocket）
- **前端**：保留 Angular + Ionic UI，通过 Tauri invoke 桥接
- **优势**：减少 UI 重写工作量，快速完成安全和性能核心改善
- **劣势**：仍依赖 JavaScript 运行时

### 方案 C：Flutter + Rust FFI

- 用 Flutter 做 UI（已有大量 Ionic 组件对应物）
- 用 Rust 编写底层 HTTP 和 WebSocket 逻辑（通过 `flutter_rust_bridge`）
- 适合已有 Flutter 经验的团队

---

## 十、参考资源

- [Tauri v2 官方文档](https://v2.tauri.app)
- [Tauri Android 开发指南](https://v2.tauri.app/distribute/android/)
- [Dioxus 官方文档](https://dioxuslabs.com)
- [reqwest 异步 HTTP 客户端](https://docs.rs/reqwest)
- [tokio-tungstenite WebSocket](https://docs.rs/tokio-tungstenite)
- [Rust 官方教程 (Chinese)](https://kaisery.github.io/trpl-zh-cn/)
- [百度人脸识别 API 文档](https://ai.baidu.com/ai-doc/FACE/yk37c1u4t)
