# RsOJ

RsOJ 是一个以 WebSocket 为主要通信方式的在线评测系统。当前有效后端位于
`rust_backend/`，由 `main_backend` 动态加载认证、WebSocket、判题、聊天、用户和控制面板模块；
根目录中的旧 Python 服务仅作为迁移遗留代码保留。

## 环境要求

- Linux（当前动态模块加载路径使用 `.so`）
- Rust nightly；开发构建使用 Cranelift
- Node.js 与 npm
- MySQL，当前代码连接 `mysql://root:123456@127.0.0.1:3306/`
- 判题所需的 `gcc`、`g++`、JDK 和 `python3`

首次运行先安装前端依赖：

```bash
cd client_web
npm install
cd ..
```

## 启动

从仓库根目录运行统一启动脚本：

```bash
./start.sh
```

脚本会构建 Rust workspace、原子部署动态模块，并同时启动后端与 Vite。默认地址：

- 主站：`http://127.0.0.1:5173`
- WebSocket/头像服务：`http://127.0.0.1:9983`
- 控制面板 HTTP API：`http://127.0.0.1:9990`
- React 控制面板：`http://127.0.0.1:5173/control-panel`

按 `Ctrl+C` 会停止前端，并让后端依次完成模块和数据库连接池清理。

常用选项：

```bash
./start.sh --release
./start.sh --skip-build
./start.sh --host 0.0.0.0 --port 5173
./start.sh --help
```

## 控制面板

控制面板没有网页端“首次生成令牌”流程，也不存在 debug 鉴权后门。至少配置一个角色令牌：

```bash
export CONTROL_PANEL_ADMIN_TOKEN='replace-with-a-long-random-token'
# 可选的低权限令牌：
export CONTROL_PANEL_PROBLEM_ADMIN_TOKEN='replace-with-another-random-token'
export CONTROL_PANEL_COMMUNITY_ADMIN_TOKEN='replace-with-another-random-token'
./start.sh
```

用户还需要先登录主站，并在数据库中拥有与令牌相同的 `admin_role`。使用管理脚本设置角色：

```bash
bash ./manage_admin.sh add alice super_admin
bash ./manage_admin.sh add bob problem_admin
bash ./manage_admin.sh list
```

完整权限和部署说明见 [CONTROL_PANEL.md](CONTROL_PANEL.md) 与
[ADMIN_ROLES.md](ADMIN_ROLES.md)。

## 验证

```bash
cd rust_backend
cargo check --workspace
cargo test --workspace --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings

cd ../client_web
npm run lint
npm run build
npm run test:e2e
```

Playwright 测试会自动启动 Vite，并模拟 WebSocket 和控制面板 HTTP API，不需要运行 MySQL
或 Rust 后端。首次执行前运行 `npx playwright install chromium`。

## 目录

- `client_web/`：React 19、TypeScript、Fluent UI、Redux、i18next 前端
- `rust_backend/`：Rust workspace 和动态模块
- `problem/`：题目测试数据与判题配置
- `module_config_rs.json`：后端模块启用状态、依赖和卸载超时
- `start.sh`：推荐的本地开发启动入口

## 当前限制

判题进程目前只有墙钟超时、虚拟内存和输出大小限制，没有使用容器、chroot、cgroup 或
seccomp。不要在面向不可信用户的公网环境中直接运行当前判题器。
