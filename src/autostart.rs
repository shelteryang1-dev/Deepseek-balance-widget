use anyhow::{Context, Result};
use winreg::enums::*;
use winreg::RegKey;

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APP_NAME: &str = "DeepSeekTray";

pub fn is_enabled() -> bool {
    get_run_key()
        .and_then(|key| Ok(key.get_value::<String, _>(APP_NAME)?))
        .is_ok()
}

pub fn enable() -> Result<()> {
    let exe_path =
        std::env::current_exe().context("failed to get executable path")?;
    let exe_str = exe_path
        .to_str()
        .context("executable path is not valid UTF-8")?;

    let key = get_or_create_run_key()?;
    key.set_value(APP_NAME, &exe_str)
        .context("failed to write Run registry key")?;

    log::info!("autostart enabled: {}", exe_str);
    Ok(())
}

pub fn disable() -> Result<()> {
    match get_run_key_write() {
        Ok(key) => match key.delete_value(APP_NAME) {
            Ok(()) => log::info!("autostart disabled"),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                log::info!("autostart entry not found, nothing to remove");
            }
            Err(e) => return Err(e).context("failed to delete Run registry value"),
        },
        Err(e) => {
            // Only ignore if Run key genuinely doesn't exist
            let msg = format!("{}", e);
            if msg.contains("系统找不到指定的文件") || msg.contains("not found") {
                log::info!("Run key not found, nothing to remove");
                return Ok(());
            }
            return Err(e).context("failed to open Run registry key");
        }
    }
    Ok(())
}

pub fn set_enabled(enabled: bool) -> Result<()> {
    if enabled { enable() } else { disable() }
}

fn get_run_key() -> Result<RegKey> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.open_subkey_with_flags(RUN_KEY, KEY_READ)
        .context("failed to open Run registry key")
}

fn get_run_key_write() -> Result<RegKey> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.open_subkey_with_flags(RUN_KEY, KEY_READ | KEY_WRITE)
        .context("failed to open Run registry key")
}

fn get_or_create_run_key() -> Result<RegKey> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.create_subkey(RUN_KEY)
        .map(|(key, _)| key)
        .context("failed to create Run registry key")
}
