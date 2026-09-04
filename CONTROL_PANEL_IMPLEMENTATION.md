# 控制面板实现说明

## 组成

- `client_web/src/router/ControlPanel.tsx`：React 管理界面
- `client_web/src/router/ProblemEditor.tsx`：题目和测试数据编辑器
- `client_web/src/controlPanelApi.ts`：API 地址、Bearer Token 和 `sessionStorage`
- `rust_backend/control_panel/src/http_server.rs`：Axum 路由和服务端授权
- `rust_backend/control_panel/src/{analytics,system_monitor,user_management,problem_management,db_admin}.rs`：业务逻辑
- `rust_backend/control_panel/src/panel_html.rs`：访问控制面板 HTTP 根路径时使用的独立内置页面

## 安全边界

令牌由环境变量提供，使用常量时间比较，不写入数据库。中间件将令牌解析为角色，并在调用
handler 前按路径检查权限。`/api/verify-admin` 还要求提交的用户名在数据库中拥有同一角色。

当前授权粒度：统计允许任意管理员角色，题目 API 允许 `problem_admin` 与 `super_admin`，系统、
用户和清库 API 只允许 `super_admin`。

## 数据与文件

- 数据库连接使用 `mysql_async::Pool`。
- 访问统计存放于 `control_panel_visits`。
- 清库逻辑跳过所有 `control_panel_` 前缀表。
- 题目主体存于 `problems`，测试输入/答案和 judge 配置同时使用根目录 `problem/`。
- WebSocket 服务通过本机 HTTP 端点上报访问量和活动连接数。

## 已知限制

- API 端口自动递增后，React 客户端需要手动通过 `VITE_CONTROL_PANEL_URL` 指向实际端口。
- 社区管理员对应的审核 API 尚未实现。
- 没有审计日志、备份恢复、封禁用户或管理员管理 API。
- 管理员角色令牌是共享凭据，不是每用户凭据。

面向使用者的完整说明见 [CONTROL_PANEL.md](CONTROL_PANEL.md)。
