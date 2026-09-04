# RsOJ 管理员角色

管理员身份由 `RsOJ.users.admin_role` 和角色令牌共同决定。数据库角色控制 React 页面显示，
Bearer Token 的角色由后端用于逐路由授权。

## 角色与当前权限

| 功能 | problem_admin | community_admin | super_admin |
| --- | --- | --- | --- |
| 查看 Dashboard | 是 | 是 | 是 |
| 管理题目和测试数据 | 是 | 否 | 是 |
| 系统监控 | 否 | 否 | 是 |
| 用户查看与删除 | 否 | 否 | 是 |
| 清空数据库业务数据 | 否 | 否 | 是 |

`community_admin` 的讨论审核接口尚未实现，目前只能查看 Dashboard。比赛、题解审核、用户禁言和
管理员管理也不属于当前控制面板功能。

## 配置角色令牌

```bash
export CONTROL_PANEL_ADMIN_TOKEN='super-admin-secret'
export CONTROL_PANEL_PROBLEM_ADMIN_TOKEN='problem-admin-secret'
export CONTROL_PANEL_COMMUNITY_ADMIN_TOKEN='community-admin-secret'
./start.sh
```

`CONTROL_PANEL_ADMIN_TOKEN` 对应 `super_admin`。只需配置实际使用的角色，但同一值不要复用于
多个角色。更改令牌后需要重启后端。

## 管理数据库角色

管理脚本需要本机 `mysql` CLI，并使用项目当前的开发数据库配置：

```bash
bash ./manage_admin.sh add alice super_admin
bash ./manage_admin.sh add bob problem_admin
bash ./manage_admin.sh add carol community_admin
bash ./manage_admin.sh remove alice
bash ./manage_admin.sh list
bash ./manage_admin.sh help
```

对应 SQL 为：

```sql
UPDATE RsOJ.users SET admin_role = 'super_admin' WHERE username = 'alice';
UPDATE RsOJ.users SET admin_role = NULL WHERE username = 'alice';
SELECT id, username, admin_role FROM RsOJ.users WHERE admin_role IS NOT NULL;
```

认证模块启动时会为旧数据库补充可空的 `admin_role VARCHAR(32)` 字段。合法值为
`problem_admin`、`community_admin`、`super_admin` 或 `NULL`。

## 访问校验

React 控制面板读取主站保存在 `localStorage` 中的用户名，再使用 Bearer Token 调用
`/api/verify-admin`。数据库角色必须与令牌角色一致。服务端随后仍会对每个 API 路径检查令牌
权限，因此绕过前端菜单不能提升权限。

这里使用的控制面板角色令牌不是主站 WebSocket session token，也不是 cookie。
