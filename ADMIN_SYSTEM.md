# 管理员系统说明（已归档）

本文件原先描述基于 `users.is_admin` 和 cookie 的旧实现，该实现已经移除。

当前系统使用 `users.admin_role` 与环境变量提供的角色令牌。请阅读：

- [ADMIN_ROLES.md](ADMIN_ROLES.md)：角色、令牌和权限矩阵
- [CONTROL_PANEL.md](CONTROL_PANEL.md)：控制面板鉴权与 HTTP API
- [CONTROL_PANEL_QUICKSTART.md](CONTROL_PANEL_QUICKSTART.md)：最短启动流程
