# RsOJ 管理员角色系统

## 概述

控制面板现在使用基于角色的访问控制（RBAC）系统，取代了原有的令牌机制。管理员分为三个角色，每个角色有不同的权限。

## 管理员角色

### 1. 题目管理员（problem_admin）
**职责**：管理题目、比赛和题解

**权限**：
- ✅ 访问 Dashboard（查看统计）
- ✅ 管理题目（创建、编辑、删除）
- ✅ 管理比赛
- ✅ 管理题解
- ❌ 无法访问系统监控
- ❌ 无法管理用户
- ❌ 无法修改系统设置

**适用人群**：出题人、题目审核员

### 2. 社区管理员（community_admin）
**职责**：管理讨论区和社区内容

**权限**：
- ✅ 访问 Dashboard（查看统计）
- ✅ 管理讨论区
- ✅ 内容审核
- ✅ 用户禁言/解禁
- ❌ 无法管理题目
- ❌ 无法访问系统监控
- ❌ 无法修改系统设置

**适用人群**：版主、内容审核员

### 3. 超级管理员（super_admin）
**职责**：管理站务和管理组内部事务

**权限**：
- ✅ **完全访问所有功能**
- ✅ Dashboard（查看所有统计）
- ✅ System Monitor（系统监控）
- ✅ Users（用户管理）
- ✅ Problems（题目管理）
- ✅ Settings（系统设置）
- ✅ 可以管理其他管理员

**适用人群**：站长、核心管理员

## 权限矩阵

| 功能 | problem_admin | community_admin | super_admin |
|------|---------------|-----------------|-------------|
| Dashboard | ✓ | ✓ | ✓ |
| 题目管理 | ✓ | ✗ | ✓ |
| 比赛管理 | ✓ | ✗ | ✓ |
| 题解管理 | ✓ | ✗ | ✓ |
| 讨论区管理 | ✗ | ✓ | ✓ |
| 系统监控 | ✗ | ✗ | ✓ |
| 用户管理 | ✗ | ✗ | ✓ |
| 系统设置 | ✗ | ✗ | ✓ |
| 管理员管理 | ✗ | ✗ | ✓ |

## 使用管理脚本

### 设置管理员

```bash
# 设置超级管理员
./manage_admin.sh add alice super_admin

# 设置题目管理员
./manage_admin.sh add bob problem_admin

# 设置社区管理员
./manage_admin.sh add charlie community_admin
```

### 移除管理员权限

```bash
./manage_admin.sh remove alice
```

### 查看所有管理员

```bash
./manage_admin.sh list
```

**输出示例**：
```
📋 Administrators by Role:
============================================================

🔴 Super Administrators (Full Access):
------------------------------------------------------------
id  username  created_at
1   alice     2026-07-23 10:30:00

🟢 Problem Administrators (Problems, Contests, Solutions):
------------------------------------------------------------
id  username  created_at
2   bob       2026-07-23 11:00:00

🔵 Community Administrators (Discussions, Moderation):
------------------------------------------------------------
id  username  created_at
3   charlie   2026-07-23 11:30:00
```

### 查看帮助

```bash
./manage_admin.sh help
```

## 数据库结构

### users 表新增字段

```sql
ALTER TABLE users ADD COLUMN admin_role VARCHAR(50) DEFAULT NULL 
COMMENT 'Admin role: problem_admin, community_admin, super_admin, or NULL for regular users';
```

**可能的值**：
- `NULL` - 普通用户（默认）
- `'problem_admin'` - 题目管理员
- `'community_admin'` - 社区管理员
- `'super_admin'` - 超级管理员

## 访问流程

1. **用户登录主站**
   - 访问 `http://localhost:5173`
   - 使用用户名和密码登录
   - 获得 `session_token` cookie

2. **访问控制面板**
   - 访问 `http://localhost:5173/control-panel`
   - 系统自动读取 `session_token`
   - 后端验证用户身份和角色

3. **权限检查**
   - 如果用户有管理员角色 → 显示对应权限的菜单
   - 如果用户没有管理员角色 → 显示"Access denied"
   - 如果用户未登录 → 提示登录

## API 端点

### POST /api/verify-admin

验证用户的管理员身份和角色。

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
  "admin_role": "super_admin",
  "username": "alice"
}
```

**字段说明**：
- `ok`: 请求是否成功
- `is_admin`: 用户是否为管理员（admin_role不为NULL）
- `admin_role`: 管理员角色（problem_admin / community_admin / super_admin）
- `username`: 用户名

## 前端实现

### 角色验证

```typescript
const checkAdminAccess = async () => {
  const sessionToken = getCookie('session_token');
  
  const response = await fetch('/api/verify-admin', {
    method: 'POST',
    body: JSON.stringify({ session_token: sessionToken }),
  });
  
  const data = await response.json();
  
  if (data.ok && data.is_admin) {
    setAdminRole(data.admin_role);
    setIsAuthenticated(true);
  }
};
```

### 权限检查

```typescript
const hasPermission = (requiredPermissions: string[]): boolean => {
  // 超级管理员有所有权限
  if (adminRole === 'super_admin') return true;
  
  // 检查当前角色是否在允许的角色列表中
  return requiredPermissions.includes(adminRole);
};
```

### 条件渲染

```tsx
{hasPermission(['problem_admin', 'super_admin']) && (
  <Button onClick={() => setActiveTab('problems')}>
    Problems
  </Button>
)}
```

## 升级指南

### 从旧系统迁移

如果你使用的是旧的 `is_admin` 字段系统：

1. **数据库已自动更新**：
   - `is_admin` 字段已删除
   - `admin_role` 字段已添加

2. **重新设置管理员**：
   ```bash
   # 将所有原来的管理员设置为超级管理员
   ./manage_admin.sh add <username> super_admin
   ```

3. **重启后端**：
   ```bash
   cd rust_backend
   ./build&run.sh
   ```

4. **测试访问**：
   - 登录主站
   - 访问控制面板
   - 验证权限正常

## 安全特性

✅ **会话验证**：使用现有的 session_token 机制
✅ **角色隔离**：不同角色只能访问其权限范围内的功能
✅ **最小权限原则**：默认所有用户都不是管理员
✅ **CORS 保护**：后端只允许来自 localhost 的请求
✅ **本地绑定**：后端只绑定到 127.0.0.1

## 常见问题

### Q: 我想让某人只管理题目，不能访问用户数据

**A**: 设置为 problem_admin：
```bash
./manage_admin.sh add username problem_admin
```

### Q: 如何修改用户的角色？

**A**: 直接重新设置即可：
```bash
# 从 problem_admin 改为 super_admin
./manage_admin.sh add username super_admin
```

### Q: 超级管理员可以做什么？

**A**: 超级管理员拥有完全访问权限，包括：
- 查看系统监控数据
- 管理所有用户
- 管理题目和讨论
- 修改系统设置
- 清空数据库

### Q: 我忘记设置超级管理员怎么办？

**A**: 使用管理脚本设置第一个超级管理员：
```bash
./manage_admin.sh add <你的用户名> super_admin
```

### Q: 访问控制面板时显示 "Access denied"

**原因**：
1. 用户没有管理员角色
2. 用户未登录

**解决方案**：
```bash
# 1. 确认用户有管理员角色
./manage_admin.sh list

# 2. 如果没有，设置角色
./manage_admin.sh add <username> <role>

# 3. 确保已登录主站
# 4. 重新访问控制面板
```

## 开发建议

### 添加新的管理功能

如果需要添加新的管理功能，请考虑以下权限分配：

- **题目相关** → `problem_admin` 和 `super_admin`
- **社区相关** → `community_admin` 和 `super_admin`
- **系统相关** → 仅 `super_admin`

**示例代码**：
```typescript
{hasPermission(['problem_admin', 'super_admin']) && (
  <Button onClick={handleNewFeature}>
    New Feature
  </Button>
)}
```

## 与原系统的区别

| 特性 | 旧系统（令牌） | 新系统（角色） |
|------|---------------|---------------|
| 认证方式 | 独立令牌 | Session Token |
| 权限粒度 | 全部或无 | 三级角色 |
| 管理工具 | 无 | manage_admin.sh |
| 开发模式 | 自动绕过 | **已移除** |
| 首次设置 | 生成令牌 | **已移除** |

**主要改进**：
- ✅ 移除了令牌机制，简化认证流程
- ✅ 基于角色的细粒度权限控制
- ✅ 集成主站的用户系统
- ✅ 更安全、更易管理
