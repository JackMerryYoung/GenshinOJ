# Control Panel

RsOJ 控制面板是一个基于 React 的管理界面，用于管理整个在线评测系统。

## 访问方式

控制面板运行在独立的 HTTP 服务器上：

- **后端地址**: `http://localhost:9990` (仅绑定到 127.0.0.1，不对外网开放)
- **前端路由**: `http://localhost:5173/control-panel` (开发模式)

## 功能特性

### 1. 仪表板 (Dashboard)
- 查看最近30天的访问统计
- 每日和累计指标展示

### 2. 系统监控 (System Monitor)
- 数据库连接状态
- 活跃的 WebSocket 连接数
- 系统运行时间
- 内存使用情况
- 数据库统计信息（用户、题目、提交、讨论总数）

### 3. 用户管理 (Users)
- 查看所有用户列表
- 搜索用户
- 查看用户统计（AC数、提交数、讨论数）
- 删除用户（级联删除相关数据）

### 4. 题目管理 (Problems)
- 查看所有题目列表
- 搜索题目
- 创建新题目
- 查看题目统计（提交数、通过数、通过率）
- 删除题目（级联删除相关提交和题解）

### 5. 系统设置 (Settings)
- 清空数据库（危险操作，保留表结构）

## 首次设置

### 生产环境

1. 首次访问控制面板时，点击 "Generate Admin Token" 生成管理员令牌
2. **重要**: 立即保存生成的令牌，它只会显示一次
3. 令牌使用 MD5 哈希存储在数据库中，原文无法恢复
4. 每次访问控制面板都需要输入令牌进行身份验证

### 开发环境

在 Debug 构建模式下（`cargo build`），身份验证被自动绕过，无需令牌即可访问所有功能。

## 安全注意事项

1. **仅本地访问**: 后端绑定到 `127.0.0.1`，不接受外网连接
2. **远程访问**: 如需远程管理，请使用 SSH 隧道：
   ```bash
   ssh -L 9990:localhost:9990 user@server
   ```
3. **令牌保管**: 管理员令牌具有完全控制权限，请妥善保管
4. **数据备份**: 执行清空数据库等危险操作前，请先备份数据

## API 端点

所有 API 端点都需要在请求体中包含 `token` 字段（开发模式除外）：

- `GET /api/status` - 检查配置状态
- `POST /api/setup` - 生成管理员令牌（仅首次）
- `POST /api/login` - 验证令牌
- `POST /api/stats` - 获取统计数据
- `POST /api/system-status` - 获取系统状态
- `POST /api/database-stats` - 获取数据库统计
- `POST /api/users` - 获取用户列表
- `POST /api/users/{id}/delete` - 删除用户
- `POST /api/problems` - 获取题目列表
- `POST /api/problems/create` - 创建题目
- `POST /api/problems/{id}/delete` - 删除题目
- `POST /api/clear-database` - 清空数据库

## 开发说明

### 前端开发

控制面板前端位于 `client_web/src/router/ControlPanel.tsx`，使用以下技术栈：

- React 19 + TypeScript
- Fluent UI 组件库
- React Router 路由
- 直接 fetch API 调用（不使用 WebSocket）

### 后端开发

控制面板后端位于 `rust_backend/control_panel/`，模块结构：

- `http_server.rs` - Axum HTTP 服务器和路由
- `auth.rs` - 令牌生成和验证
- `analytics.rs` - 访问统计分析
- `system_monitor.rs` - 系统监控
- `user_management.rs` - 用户管理
- `problem_management.rs` - 题目管理
- `db_admin.rs` - 数据库管理操作

### 题目创建

创建题目时需要提供：

- **Problem Number** (必填): 唯一的题目编号
- **Problem Name** (必填): 题目名称
- **Difficulty** (必填): 难度级别
  - 1: Easy
  - 2: Medium
  - 3: Hard
  - 4: Expert
- **Problem Statement** (必填): 题目描述（支持 Markdown）
- **Testcase Config** (可选): 测试用例配置（JSON 格式）

## 故障排查

### 无法连接到控制面板

检查后端模块是否正常启动：
```bash
cd rust_backend
./build&run.sh
```

查看日志中是否有 `[CONTROL_PANEL] [INFO] Control panel listening on http://127.0.0.1:9990` 消息。

### 端口被占用

控制面板会自动尝试更高的端口号，查看启动日志确认实际使用的端口。

### 忘记管理员令牌

生产环境中，如果丢失令牌，需要手动删除数据库中的记录：
```sql
DELETE FROM RsOJ.control_panel_admin;
```

然后重新访问控制面板生成新令牌。

## 数据库表

控制面板使用两个独立的表（带 `control_panel_` 前缀）：

- `control_panel_admin` - 存储管理员令牌哈希
- `control_panel_visits` - 记录网站访问日志

这些表在执行"清空数据库"操作时会被保留。
