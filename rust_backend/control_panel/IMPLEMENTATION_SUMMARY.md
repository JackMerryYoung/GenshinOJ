# 控制面板实现状态

本文件只记录当前实现边界，不再维护历史代码量和已经失效的 schema/API 清单。

- 控制面板是独立 Axum HTTP 模块，同时提供 React 页面所需 API 和一个内置 HTML 页面。
- 数据库 schema 以当前 Rust 模块启动时创建或迁移的表为准。
- 鉴权令牌来自环境变量，不存入数据库，不存在 debug 绕过或网页生成流程。
- 服务端实现 `super_admin`、`problem_admin`、`community_admin` 三种令牌角色。
- React 客户端通过 `sessionStorage` 保存控制面板令牌，通过 `localStorage` 读取主站用户名。
- 当前只有统计、系统监控、用户列表/删除、题目 CRUD/上传和清库功能。
- 管理端到端流程由 `client_web/e2e/control-panel.spec.ts` 覆盖。

详细资料：

- [`README.md`](README.md)：模块开发入口
- [`../../CONTROL_PANEL.md`](../../CONTROL_PANEL.md)：当前 API 与部署
- [`../../ADMIN_ROLES.md`](../../ADMIN_ROLES.md)：角色模型
