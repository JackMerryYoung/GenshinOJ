# RsOJ Control Panel - 快速启动指南

## 新功能概览

✅ **系统监控面板**
- 实时系统状态（数据库、WS连接、内存、运行时间）
- 数据库统计（用户数、题目数、提交数、数据库大小）

✅ **用户管理**
- 用户列表查看和搜索
- 封禁/解封用户
- 删除用户
- 查看用户详细信息和统计

✅ **题目管理**
- 题目列表查看和搜索
- 创建新题目
- 编辑题目信息
- 显示/隐藏题目
- 删除题目
- 查看提交统计和通过率

## 编译状态

✅ 编译成功！只有少量不影响功能的警告。

## 如何使用

### 1. 编译并运行
```bash
cd /home/jackmerryyoung/RsOJ/rust_backend
cargo build --release
# 运行你的后端服务器
```

### 2. 访问控制面板
- 控制面板会在一个独立的 HTTP 端口上运行
- 查看终端输出找到类似这样的消息：
  ```
  Control panel listening on http://127.0.0.1:XXXX
  ```
- 在浏览器中打开该地址

### 3. 首次设置
1. 首次访问会看到"First-time setup"界面
2. 点击"Generate admin token"生成管理员令牌
3. **重要**：复制并保存这个令牌（只显示一次！）
4. 点击"I saved it"进入登录界面

### 4. 登录
- 输入保存的 admin token
- 点击"Unlock"解锁控制面板

### 5. 使用新功能

#### 系统监控
- 点击左侧"System Monitor"查看：
  - 数据库连接状态
  - 活跃 WebSocket 连接数
  - 服务器运行时间
  - 内存使用情况
  - 数据库统计（用户数、题目数等）

#### 用户管理
- 点击左侧"Users"进入用户管理
- 使用搜索框查找用户
- 可以封禁/解封/删除用户
- 查看每个用户的提交数和状态

#### 题目管理
- 点击左侧"Problems"进入题目管理
- 点击"Create Problem"创建新题目
- 使用搜索框查找题目
- 可以显示/隐藏/删除题目
- 查看每个题目的通过率和统计

## 文件结构

新增的文件：
```
rust_backend/control_panel/src/
├── system_monitor.rs      # 系统监控逻辑
├── user_management.rs     # 用户管理逻辑
├── problem_management.rs  # 题目管理逻辑
├── panel_html.rs         # 前端界面（HTML/CSS/JS）
└── FEATURES.md           # 详细功能文档
```

修改的文件：
```
rust_backend/control_panel/
├── src/lib.rs            # 添加新模块引用
├── src/http_server.rs    # 添加新 API 路由
└── Cargo.toml            # 添加 lazy_static 依赖
```

## API 端点一览

### 系统监控
- `POST /api/system-status` - 获取系统状态
- `POST /api/database-stats` - 获取数据库统计

### 用户管理
- `POST /api/users` - 用户列表（分页+搜索）
- `POST /api/users/:id` - 用户详情
- `POST /api/users/:id/ban` - 封禁用户
- `POST /api/users/:id/unban` - 解封用户
- `POST /api/users/:id/delete` - 删除用户

### 题目管理
- `POST /api/problems` - 题目列表（分页+搜索）
- `POST /api/problems/create` - 创建题目
- `POST /api/problems/:id` - 题目详情
- `POST /api/problems/:id/update` - 更新题目
- `POST /api/problems/:id/toggle-visibility` - 切换可见性
- `POST /api/problems/:id/delete` - 删除题目

## 注意事项

⚠️ **安全提示**
- Admin token 使用 MD5+salt 哈希存储
- 会话在30分钟无操作后过期
- 所有破坏性操作都需要确认

⚠️ **数据库操作**
- 删除用户会级联删除其提交和讨论
- 删除题目会级联删除相关提交
- 清空数据库会保留控制面板的管理员令牌

## 已知限制

1. **数据库连接池大小** - 当前使用固定值（mysql_async 在此版本不提供 status 方法）
2. **WebSocket 连接计数** - 需要在 ws_server 模块中集成才能正确统计
3. **题目创建** - 前端界面简化版，可能需要扩展更多字段

## 下一步建议

1. **集成 WebSocket 计数器** - 在 ws_server 中调用 control_panel 的计数函数
2. **添加审计日志** - 记录所有管理员操作
3. **题目批量导入** - 支持从 JSON/YAML 批量导入题目
4. **评测队列监控** - 实时查看评测任务状态

## 获取帮助

详细文档请查看：
- `FEATURES.md` - 完整功能文档
- 源代码注释 - 每个函数都有详细说明

## 测试建议

1. **系统监控** - 确保显示正确的统计数据
2. **用户管理** - 测试搜索、封禁、删除功能
3. **题目管理** - 测试创建、编辑、显示/隐藏功能
4. **权限控制** - 确认未登录时无法访问 API

---

祝使用愉快！如有问题，请查看日志或联系开发团队。
