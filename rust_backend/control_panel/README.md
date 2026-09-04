# control_panel module

`control_panel` 是由 `main_backend` 动态加载的 Axum HTTP 管理模块。面向使用者的权威文档位于：

- [`../../CONTROL_PANEL.md`](../../CONTROL_PANEL.md)
- [`../../ADMIN_ROLES.md`](../../ADMIN_ROLES.md)
- [`../../CONTROL_PANEL_QUICKSTART.md`](../../CONTROL_PANEL_QUICKSTART.md)

## 开发入口

- `src/http_server.rs`：路由、Bearer Token 认证和角色授权
- `src/global.rs`：数据库、监听地址和角色令牌配置
- `src/analytics.rs`：30 天统计序列和访问记录
- `src/system_monitor.rs`：系统与数据库指标
- `src/user_management.rs`：用户列表和删除
- `src/problem_management.rs`：题目 CRUD 与 testcase 配置同步
- `src/db_admin.rs`：清空业务表
- `src/panel_html.rs`：API 根路径提供的独立内置页面

## 本地验证

```bash
cd rust_backend
cargo test -p control_panel
cargo clippy -p control_panel --all-targets -- -D warnings
```

完整系统建议从仓库根目录使用 `./start.sh` 启动。控制面板默认绑定 `127.0.0.1:9990`；
令牌通过 `CONTROL_PANEL_ADMIN_TOKEN`、`CONTROL_PANEL_PROBLEM_ADMIN_TOKEN` 和
`CONTROL_PANEL_COMMUNITY_ADMIN_TOKEN` 提供。
