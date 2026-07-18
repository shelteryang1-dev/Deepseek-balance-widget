# DeepSeek Tray

显示 DeepSeek API 余额在 Windows 右下角系统托盘。

## 使用方式

### 下载即用

从 [Releases](../../releases) 下载 `deepseek-tray.exe`，双击运行。

### 配置 API Key

三种方式任选其一：

1. **环境变量**：`set DEEPSEEK_API_KEY=sk-xxxxxxxx`
2. **配置文件**：右键托盘图标 → "设置 API Key" → 记事本编辑 `config.toml`
3. **手动创建**：在 `%APPDATA%\deepseek-tray\config.toml` 写入：

```toml
api_key = "sk-xxxxxxxx"
refresh_interval_minutes = 30
auto_start = false
```

## 功能

- 托盘图标显示余额整数（白色文字，透明背景）
- 悬停查看详细信息（充值余额 / 赠送余额）
- 右键菜单：刷新、复制余额、切换刷新间隔（15/30/60分钟）、设置 API Key、开机自启、退出
- DPI 自适应（100%～200% 清晰显示）

## 编译

需要 [Rust](https://rustup.rs)：

```bash
cargo build --release
# 输出: target/release/deepseek-tray.exe
```

## License

MIT
