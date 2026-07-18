use anyhow::{Context, Result};
use winreg::enums::*;
use winreg::RegKey;

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APP_NAME: &str = "DeepSeekTray";

/// Check if autostart is currently enabled in the registry.
pub fn is_enabled() -> bool {
    get_run_key()
        .and_then(|key| {
            let value: String = key.get_value(APP_NAME)?;
            Ok(value)
        })
        .is_ok()
}

/// Add the current executable to the Windows Run registry key.
pub fn enable() -> Result<()> {
    let exe_path = std::env::current_exe()
        .context("无法获取当前可执行文件路径")?;
    let exe_str = exe_path
        .to_str()
        .context("可执行文件路径包含非 UTF-8 字符")?;

    let key = get_or_create_run_key()?;
    key.set_value(APP_NAME, &exe_str)
        .context("写入注册表 Run 键失败")?;

    log::info!("已启用开机自启: {}", exe_str);
    Ok(())
}

/// Remove the current executable from the Windows Run registry key.
pub fn disable() -> Result<()> {
    let run_key = get_run_key_write();
    match run_key {
        Ok(key) => {
            match key.delete_value(APP_NAME) {
                Ok(()) => log::info!("已禁用开机自启"),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    log::info!("开机自启未启用（无注册表项）");
                }
                Err(e) => return Err(e).context("删除注册表 Run 键失败"),
            }
        }
        Err(_) => {
            // Key doesn't exist, nothing to remove
        }
    }
    Ok(())
}

/// Toggle autostart state.
pub fn set_enabled(enabled: bool) -> Result<()> {
    if enabled {
        enable()
    } else {
        disable()
    }
}

fn get_run_key() -> Result<RegKey> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.open_subkey_with_flags(RUN_KEY, KEY_READ).context("打开注册表 Run 键失败")
}

fn get_run_key_write() -> Result<RegKey> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.open_subkey_with_flags(RUN_KEY, KEY_READ | KEY_WRITE).context("打开注册表 Run 键失败")
}

fn get_or_create_run_key() -> Result<RegKey> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.create_subkey(RUN_KEY)
        .map(|(key, _)| key)
        .context("创建注册表 Run 键失败")
}
