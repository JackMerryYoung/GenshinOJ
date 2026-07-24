# 控制面板快速开始指南

## 启动步骤

### 1. 启动后端

```bash
cd rust_backend
./build&run.sh
```

等待所有模块加载完成，你应该看到类似的日志：

```
[CONTROL_PANEL] [INFO] [THREAD xxx] [FILE `src/http_server.rs` LINE xxx] Control panel listening on http://127.0.0.1:9990
```

### 2. 启动前端开发服务器

在另一个终端：

```bash
cd client_web
npm run dev
```

前端将运行在 `http://localhost:5173`

### 3. 访问控制面板

打开浏览器，访问：

```
http://localhost:5173/control-panel
```

### 4. 首次设置（生产环境）

如果是第一次访问且使用 release 构建：

1. 点击 "Generate Admin Token" 按钮
2. 保存生成的令牌（它只显示一次！）
3. 使用令牌登录

### 5. 开发模式

如果使用 debug 构建（`./debug.sh`），身份验证会自动绕过，可以直接访问所有功能。

## 功能测试

### 测试仪表板
- 访问 Dashboard 标签页查看统计数据
- 统计数据来自最近 30 天的活动

### 测试系统监控
1. 点击 "System Monitor" 标签
2. 查看实时系统状态（每5秒自动刷新）
3. 查看数据库统计信息

### 测试用户管理
1. 点击 "Users" 标签
2. 在搜索框中输入用户名进行搜索
3. 查看用户统计数据
4. 可以删除测试用户（谨慎使用）

### 测试题目管理
1. 点击 "Problems" 标签
2. 点击 "Create Problem" 创建新题目
3. 填写表单：
   - Problem Number: 例如 1001
   - Problem Name: 例如 "A+B Problem"
   - Difficulty: 选择难度级别
   - Problem Statement: 使用 Markdown 编写题目描述
   - Testcase Config: 可选的 JSON 配置
4. 查看题目列表和统计数据
5. 可以删除测试题目

### 测试数据库清空
1. 点击 "Settings" 标签
2. 点击 "Clear Database" 按钮
3. 确认两次警告
4. 数据库中的所有数据将被清空（保留表结构和控制面板相关表）

## 常见问题

### Q: 无法连接到控制面板
A: 确保后端已启动且端口 9990 未被占用。检查后端日志确认控制面板是否成功启动。

### Q: CORS 错误
A: 控制面板后端绑定在 localhost，前端也在 localhost，不应该有 CORS 问题。如果遇到，检查浏览器控制台的详细错误信息。

### Q: 统计数据为空
A: 新安装的系统可能没有历史数据。使用系统一段时间后（提交代码、发布讨论等），统计数据会逐渐积累。

### Q: 忘记管理员令牌
A: 在生产环境中，如果丢失令牌，需要手动删除数据库记录：
```sql
DELETE FROM RsOJ.control_panel_admin;
```
然后重新生成新令牌。

## 技术架构

### 前端
- **框架**: React 19 + TypeScript
- **UI 库**: Fluent UI (@fluentui/react-components)
- **路由**: React Router
- **状态**: 本地 useState（不使用 Redux）
- **HTTP 客户端**: 原生 fetch API

### 后端
- **Web 框架**: Axum (异步 HTTP)
- **数据库**: MySQL (mysql_async)
- **身份验证**: MD5 哈希令牌
- **监控**: 实时系统指标收集

### 通信协议
- 控制面板使用传统的 HTTP REST API
- 与主应用的 WebSocket 通信完全独立
- 所有 API 请求都需要在 body 中包含 token（开发模式除外）

## 开发提示

### 添加新的 API 端点

1. 在 `rust_backend/control_panel/src/http_server.rs` 添加路由：
```rust
.route("/api/new-endpoint", axum::routing::post(new_handler))
```

2. 实现处理函数：
```rust
async fn new_handler(
    axum::Json(request): axum::Json<TokenRequest>
) -> (axum::http::StatusCode, axum::Json<ApiResponse>) {
    // 验证 token
    match crate::auth::verify_token(&request.token).await {
        Ok(true) => {}
        Ok(false) => return unauthorized_response(),
        Err(e) => return internal_error(e),
    }
    
    // 实现业务逻辑
    // ...
}
```

3. 在前端调用：
```typescript
const response = await fetch(`${CONTROL_PANEL_URL}/api/new-endpoint`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ token }),
});
```

### 添加新的前端标签页

1. 在 `ControlPanel.tsx` 的侧边栏添加按钮
2. 在内容区域添加条件渲染
3. 创建新的组件函数（例如 `MyNewTab`）
4. 实现数据获取和渲染逻辑

## 性能优化建议

1. **系统监控**: 已配置为每 5 秒自动刷新，可以根据需要调整间隔
2. **分页**: 用户和题目列表都支持分页，默认每页 10 条
3. **搜索**: 使用 SQL LIKE 查询，建议在大数据量时添加索引
4. **缓存**: 统计数据可以考虑添加短期缓存减少数据库查询

## 安全检查清单

- [x] 后端仅绑定到 127.0.0.1
- [x] 令牌使用哈希存储，不存储明文
- [x] 开发模式绕过仅在 debug 构建中启用
- [x] 危险操作（删除、清空）需要多次确认
- [x] 控制面板表不会被"清空数据库"操作影响
