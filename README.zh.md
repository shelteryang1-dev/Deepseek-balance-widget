# DeepSeek Tray

在 Windows 系统托盘显示 [DeepSeek](https://deepseek.com) API 余额。

基于 **Rust** 开发 — 极低资源占用（约 8 MB 内存），单个 `.exe` 文件，零运行时依赖。非 Electron，不臃肿。

[English](README.md)

## 下载

从 [Releases](../../releases) 下载 `deepseek-tray.exe`，双击运行。

## 配置 API Key

**推荐方式**：右键托盘图标 → **"设置 API Key"** → 记事本编辑配置文件 → 保存后点击"刷新余额"。

也可设置环境变量：

```cmd
set DEEPSEEK_API_KEY=sk-xxxxxxxx
```

或手动创建 `%APPDATA%\deepseek-tray\config.toml`：

```toml
api_key = "sk-xxxxxxxx"
refresh_interval_minutes = 30
auto_start = false
```

## 功能

- 托盘图标白色数字显示余额（透明背景）
- 悬停显示详细信息（充值余额 / 赠送余额）
- 右键菜单：刷新余额、复制余额、切换刷新间隔（15/30/60分钟）、设置 API Key、开机自启、退出
- DPI 自适应 — 100% 到 200%+ 缩放均清晰

## 开机自启

右键菜单勾选 **"开机自启"** 即可。程序会在注册表 `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` 写入启动项，取消勾选自动删除。

## 编译（供开发者）

需要 [Rust](https://rustup.rs)：

```bash
cargo build --release
# 输出: target/release/deepseek-tray.exe（约 4 MB）
```

## 开源协议

MIT
