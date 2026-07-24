# 控制面板管理员权限系统

## 概述

控制面板现在需要管理员权限才能访问。只有被标记为管理员的用户才能进入控制面板。

## 权限验证流程

1. **用户登录检查**：检查用户是否已登录（通过session_token cookie）
2. **管理员验证**：向后端API发送session_token，验证该用户是否为管理员
3. **访问控制**：
   - 如果用户是管理员 → 进入控制面板
   - 如果用户不是管理员 → 显示"Access denied: Administrator privileges required"
   - 如果用户未登录 → 显示"You must be logged in to access the control panel"

## 设置管理员

### 方法1: 使用管理脚本（推荐）

```bash
# 设置用户为管理员
./manage_admin.sh add <username>

# 移除管理员权限
./manage_admin.sh remove <username>

# 列出所有管理员
./manage_admin.sh list
```

**示例**：
```bash
# 设置 alice 为管理员
./manage_admin.sh add alice
# 输出: ✅ User 'alice' is now an administrator

# 查看管理员列表
./manage_admin.sh list
# 输出:
# 📋 Administrators:
# ------------------------------------------------------------
# id  username  created_at
# 1   alice     2026-07-23 10:30:00
```

### 方法2: 直接使用SQL

```sql
-- 设置用户为管理员
UPDATE RsOJ.users SET is_admin = 1 WHERE username = 'alice';

-- 移除管理员权限
UPDATE RsOJ.users SET is_admin = 0 WHERE username = 'alice';

-- 查看所有管理员
SELECT id, username, created_at FROM RsOJ.users WHERE is_admin = 1;
```

## 数据库更改

添加了新的字段到 `users` 表：

```sql
ALTER TABLE users ADD COLUMN is_admin TINYINT(1) NOT NULL DEFAULT 0;
```

- `is_admin = 1`：管理员
- `is_admin = 0`：普通用户（默认）

## 后端API

### 新增端点：`POST /api/verify-admin`

**请求**：
```json
{
  "session_token": "用户的session_token"
}
```

**响应**：
```json
{
  "ok": true,
  "is_admin": true,
  "username": "alice"
}
```

**字段说明**：
- `ok`: 请求是否成功
- `is_admin`: 用户是否为管理员
- `username`: 用户名（仅在ok为true时返回）

## 前端实现

控制面板组件在加载时会：

1. 从cookie中读取 `session_token`
2. 调用 `/api/verify-admin` 验证管理员身份
3. 根据验证结果决定是否显示控制面板

**相关代码**：
```typescript
const checkAdminAccess = async () => {
  const sessionToken = getCookie('session_token');
  
  if (!sessionToken) {
    setError('You must be logged in to access the control panel');
    return;
  }

  const response = await fetch(`${CONTROL_PANEL_URL}/api/verify-admin`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ session_token: sessionToken }),
  });
  
  const data = await response.json();
  
  if (data.ok && data.is_admin) {
    // 允许访问
  } else {
    setError('Access denied: Administrator privileges required');
  }
};
```

## 使用流程

### 1. 创建管理员账户

```bash
# 注册一个新账户（通过网页 /register）
# 然后使用管理脚本设置为管理员
./manage_admin.sh add your_username
```

### 2. 访问控制面板

1. 在浏览器中登录 RsOJ 主站
2. 访问 `http://localhost:5173/control-panel`
3. 如果你是管理员，将自动进入控制面板
4. 如果不是管理员，会看到错误提示

### 3. 管理其他管理员

使用 `manage_admin.sh` 脚本可以：
- 添加新的管理员
- 移除现有管理员的权限
- 查看所有管理员列表

## 安全特性

✅ **Cookie验证**：使用现有的session_token机制，不需要额外的认证
✅ **数据库验证**：每次访问都会查询数据库确认管理员状态
✅ **最小权限原则**：默认所有用户都不是管理员
✅ **CORS保护**：后端只允许来自localhost的请求
✅ **本地绑定**：后端只绑定到127.0.0.1，不对外网开放

## 故障排查

### 问题：Access denied

**原因**：
1. 用户不是管理员
2. session_token已过期
3. 用户未登录

**解决方案**：
```bash
# 1. 确认用户是管理员
./manage_admin.sh list

# 2. 如果不是，设置为管理员
./manage_admin.sh add <username>

# 3. 重新登录主站
# 4. 再次访问控制面板
```

### 问题：You must be logged in

**原因**：没有session_token cookie

**解决方案**：
1. 访问主站 `http://localhost:5173`
2. 登录你的管理员账户
3. 再访问控制面板

### 问题：Failed to verify admin access

**原因**：后端未启动或数据库连接失败

**解决方案**：
```bash
# 重启后端（加载更新的control_panel模块）
cd rust_backend
./build&run.sh
```

## 开发模式注意事项

在开发模式（debug构建）下：
- 控制面板的令牌验证仍然被绕过
- 但是管理员权限检查**不会被绕过**
- 必须是管理员才能访问，即使在开发模式下

## 迁移指南

如果你已经有现有的用户，需要：

1. **数据库已自动更新**：`is_admin`字段已添加
2. **设置第一个管理员**：
   ```bash
   ./manage_admin.sh add <你的用户名>
   ```
3. **重启后端**：加载更新的control_panel模块
4. **测试访问**：登录后访问控制面板

## 与原有令牌系统的关系

控制面板现在有**两层安全机制**：

1. **管理员验证**（新增）：
   - 验证用户是否为管理员
   - 使用session_token
   - 在每次访问控制面板时检查

2. **控制面板令牌**（原有）：
   - 生产模式下需要输入令牌
   - 开发模式下自动绕过
   - 用于保护控制面板的管理操作

**两者都需要通过**才能完全访问控制面板。
