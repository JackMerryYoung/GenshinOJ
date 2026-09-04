# RsOJ 控制面板

控制面板由两部分组成：

- React 页面：`http://127.0.0.1:5173/control-panel`
- 独立 Axum API：默认监听 `http://127.0.0.1:9990`

API 只绑定回环地址。远程管理应使用 SSH 隧道或可信反向代理，不要直接暴露端口。

## 启动与配置

控制面板令牌只从进程环境读取，不写入数据库，也不会由网页生成。按需配置：

```bash
# super_admin，拥有全部控制面板权限
export CONTROL_PANEL_ADMIN_TOKEN='replace-with-a-long-random-token'

# problem_admin，只允许统计和题目管理
export CONTROL_PANEL_PROBLEM_ADMIN_TOKEN='replace-with-another-random-token'

# community_admin，目前只允许统计；社区审核 API 尚未实现
export CONTROL_PANEL_COMMUNITY_ADMIN_TOKEN='replace-with-another-random-token'

./start.sh
```

至少需要一个非空令牌，否则受保护 API 返回 `503 Service Unavailable`。debug 和 release 构建
执行相同鉴权，不存在开发后门。

如果 React 前端不应连接默认 API 地址，在启动 Vite 前设置：

```bash
export VITE_CONTROL_PANEL_URL='http://127.0.0.1:9990'
```

跨源开发地址除默认的 `localhost:5173` 和 `127.0.0.1:5173` 外，可再允许一个来源：

```bash
export CONTROL_PANEL_CORS_ORIGIN='https://admin.example.com'
```

## 登录模型

进入 React 控制面板需要同时满足：

1. 浏览器中存在主站登录后保存的 `loginUsername`。
2. 用户表的 `admin_role` 为 `super_admin`、`problem_admin` 或 `community_admin`。
3. 输入与该数据库角色对应的控制面板令牌。

React 客户端只在当前标签页的 `sessionStorage` 中保存控制面板令牌。所有受保护请求通过
`Authorization: Bearer <token>` 发送；权限最终由服务端中间件执行，而不是依赖隐藏菜单。

控制面板令牌属于共享角色凭据，不是主站 session token。两者不要混用。

直接访问控制面板 API 根路径得到的内置 HTML 页面只验证角色令牌，不读取主站用户名。该页面
保留全部导航项，低权限令牌访问越权接口时会收到服务端 `403 Forbidden`；需要按角色隐藏菜单
和中英文界面时使用 React 控制面板。

## 权限

| 功能/API | problem_admin | community_admin | super_admin |
| --- | --- | --- | --- |
| Dashboard、`/api/stats` | 是 | 是 | 是 |
| 题目读取、创建、编辑、删除、测试数据上传 | 是 | 否 | 是 |
| 系统与数据库监控 | 否 | 否 | 是 |
| 用户列表与删除 | 否 | 否 | 是 |
| 清空业务数据 | 否 | 否 | 是 |

角色设置方法见 [ADMIN_ROLES.md](ADMIN_ROLES.md)。

## 当前功能

- 最近 30 天的每日及累计统计
- 数据库、WebSocket 连接数、运行时间、内存监控
- 用户分页、搜索、统计和删除
- 题目分页、搜索、创建、编辑、删除
- Markdown/KaTeX 题面预览
- 测试数据上传和 testcase 配置编辑
- 清空业务表，同时保留 `control_panel_` 前缀的控制面板表
- 中英文界面

尚未实现用户封禁、比赛管理、社区内容审核、管理员管理、备份恢复和审计日志。

## HTTP API

公开但只监听本机回环的端点：

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/` | 内置的独立管理页面 |
| GET | `/api/status` | 返回是否配置了任意角色令牌 |
| POST | `/api/setup` | 已禁用，固定返回 `501` |
| POST | `/api/login` | 验证任意已配置角色令牌 |
| POST | `/api/record-visit` | WebSocket 服务上报访问 |
| POST | `/api/record-ws-connection` | WebSocket 服务上报连接数变化 |

受 Bearer Token 保护的端点：

| 方法 | 路径 | 最低角色 |
| --- | --- | --- |
| POST | `/api/verify-admin` | 任意管理员角色 |
| POST | `/api/stats` | 任意管理员角色 |
| POST | `/api/system-status` | super_admin |
| POST | `/api/database-stats` | super_admin |
| POST | `/api/users` | super_admin |
| POST | `/api/users/{id}/delete` | super_admin |
| POST | `/api/problems` | problem_admin |
| GET | `/api/problems/{id}` | problem_admin |
| PUT | `/api/problems/{id}` | problem_admin |
| POST | `/api/problems/create` | problem_admin |
| POST | `/api/problems/{id}/delete` | problem_admin |
| POST | `/api/upload-testdata` | problem_admin |
| POST | `/api/clear-database` | super_admin |

## 题目和测试数据

难度取值为 `0..7`。题面在数据库中保存为 JSON 字符串数组。每个 testcase 包含：

```json
{
  "number": 1,
  "score": 100,
  "input": "1.in",
  "answer": "1.out",
  "time_limit": 1.0,
  "memory_limit": 256
}
```

`time_limit` 单位为秒，`memory_limit` 单位为 MB。上传文件分别写入
`problem/<number>/input/` 和 `problem/<number>/answer/`；编辑题目时 testcase 配置还会同步到
`problem/<number>/problem_testcase_config.json`。

## 端口说明

控制面板若无法绑定 `9990` 会尝试更高端口，但 React 客户端不会自动发现这个变化。开发时应
保证 `9990` 可用；若实际端口改变，需要同步设置 `VITE_CONTROL_PANEL_URL` 并重启 Vite。
