# DeepSeek Tray

Display your [DeepSeek](https://deepseek.com) API balance in the Windows system tray.

Built with **Rust** — minimal resource usage (~8 MB RAM), single `.exe` with zero runtime dependencies. No Electron, no bloat.

[中文说明](README.zh.md)

## Download

Get `deepseek-tray.exe` from [Releases](../../releases). Double-click to run.

## Setup API Key

**Recommended:** right-click the tray icon → **"设置 API Key"** → edit the config file in Notepad, save, then click "刷新余额".

Alternatively, set the environment variable:

```cmd
set DEEPSEEK_API_KEY=sk-xxxxxxxx
```

Or manually create `%APPDATA%\deepseek-tray\config.toml`:

```toml
api_key = "sk-xxxxxxxx"
refresh_interval_minutes = 30
auto_start = false
```

## Features

- Balance shown as white text on the tray icon (no background)
- Hover tooltip shows topped-up / granted breakdown
- Right-click menu: Refresh, Copy Balance, set refresh interval (15/30/60 min), Set API Key, Start with Windows, Quit
- DPI-aware rendering — crisp on 100% to 200%+ scaling

## Start with Windows

Enable **"开机自启"** in the right-click menu. The app writes a registry key under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`. Disabling removes it.

## Build (for developers)

Requires [Rust](https://rustup.rs):

```bash
cargo build --release
# output: target/release/deepseek-tray.exe (~4 MB)
```

## License

MIT
