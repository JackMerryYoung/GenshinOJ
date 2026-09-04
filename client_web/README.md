# RsOJ Web Client

React 19 + TypeScript + Vite 前端。页面通过 `/wsapi` 与 Rust WebSocket 服务通信；头像请求
通过 `/avatar` 代理；控制面板通过独立 HTTP API 通信。

## 开发

推荐从仓库根目录运行 `./start.sh`，它会同时启动前后端。只启动前端时：

```bash
npm install
npm run dev
```

默认 Vite 地址为 `http://127.0.0.1:5173`。开发代理配置位于 `vite.config.ts`：

- `/wsapi` -> `ws://localhost:9983/ws`
- `/avatar` -> `http://localhost:9983`

控制面板 API 默认使用 `http://localhost:9990`，可在启动 Vite 前通过
`VITE_CONTROL_PANEL_URL` 覆盖。

## 检查与构建

```bash
npm run lint
npm run build
```

## Playwright

首次安装浏览器：

```bash
npx playwright install chromium
```

运行端到端测试：

```bash
npm run test:e2e
npm run test:e2e:headed
```

Playwright 会自动启动独立的 Vite 实例，并 mock WebSocket 与控制面板 HTTP 协议，因此不依赖
Rust 后端或 MySQL。当前覆盖登录、session 恢复、中英文切换、题目搜索分页、提交主从视图和
控制面板角色访问。
