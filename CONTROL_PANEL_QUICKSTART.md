# 控制面板快速开始

## 1. 准备账户

先在主站注册用户，然后设置与准备使用的令牌相同的角色：

```bash
bash ./manage_admin.sh add alice super_admin
```

## 2. 配置令牌并启动

```bash
export CONTROL_PANEL_ADMIN_TOKEN='replace-with-a-long-random-token'
./start.sh
```

也可以使用受限角色令牌：

```bash
export CONTROL_PANEL_PROBLEM_ADMIN_TOKEN='replace-with-another-random-token'
bash ./manage_admin.sh add alice problem_admin
./start.sh
```

## 3. 访问

1. 在 `http://127.0.0.1:5173/login` 登录主站。
2. 打开 `http://127.0.0.1:5173/control-panel`。
3. 输入对应角色的控制面板令牌。

若未配置令牌，管理 API 会返回 503；系统不会从网页生成令牌。控制面板默认只监听
`127.0.0.1:9990`，远程访问请使用 SSH 隧道：

```bash
ssh -L 5173:localhost:5173 -L 9990:localhost:9990 user@server
```

完整说明见 [CONTROL_PANEL.md](CONTROL_PANEL.md)。
