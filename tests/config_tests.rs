use deepseek_tray::config::Config;
use std::env;
use std::fs;
use std::io::Write;
use std::sync::Mutex;
use tempfile::TempDir;

static ENV_MUTEX: Mutex<()> = Mutex::new(());

fn write_config(dir: &TempDir, content: &str) {
    let config_dir = dir.path().join("deepseek-tray");
    fs::create_dir_all(&config_dir).unwrap();
    let mut f = fs::File::create(config_dir.join("config.toml")).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}

#[test]
fn test_default_config() {
    let config = Config::default();
    assert_eq!(config.api_key, None);
    assert_eq!(config.refresh_interval_minutes, 30);
    assert!(!config.auto_start);
}

#[test]
fn test_load_from_file() {
    let dir = TempDir::new().unwrap();
    write_config(&dir, r#"
api_key = "sk-test-key-123"
refresh_interval_minutes = 15
auto_start = true
"#);
    let config_path = dir.path().join("deepseek-tray").join("config.toml");
    let config = Config::load_from_path(&config_path).unwrap();
    assert_eq!(config.api_key.unwrap(), "sk-test-key-123");
    assert_eq!(config.refresh_interval_minutes, 15);
    assert!(config.auto_start);
}

#[test]
fn test_load_partial_config_uses_defaults() {
    let dir = TempDir::new().unwrap();
    write_config(&dir, r#"
api_key = "sk-test-key-456"
"#);
    let config_path = dir.path().join("deepseek-tray").join("config.toml");
    let config = Config::load_from_path(&config_path).unwrap();
    assert_eq!(config.api_key.unwrap(), "sk-test-key-456");
    assert_eq!(config.refresh_interval_minutes, 30); // default
    assert!(!config.auto_start); // default
}

#[test]
fn test_save_and_reload_roundtrip() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("deepseek-tray").join("config.toml");

    let config = Config {
        api_key: Some("sk-roundtrip".into()),
        refresh_interval_minutes: 60,
        auto_start: true,
    };
    config.save_to_path(&config_path).unwrap();

    let loaded = Config::load_from_path(&config_path).unwrap();
    assert_eq!(loaded.api_key.unwrap(), "sk-roundtrip");
    assert_eq!(loaded.refresh_interval_minutes, 60);
    assert!(loaded.auto_start);
}

#[test]
fn test_resolve_api_key_from_env() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let dir = TempDir::new().unwrap();
    write_config(&dir, r#"api_key = "sk-from-file""#);
    let config_path = dir.path().join("deepseek-tray").join("config.toml");

    // Set env var — should take priority
    env::set_var("DEEPSEEK_API_KEY", "sk-from-env");
    let mut config = Config::load_from_path(&config_path).unwrap();
    let key = config.resolve_api_key(&config_path, None).unwrap();
    assert_eq!(key, "sk-from-env");
    env::remove_var("DEEPSEEK_API_KEY");
}

#[test]
fn test_resolve_api_key_from_file_when_no_env() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let dir = TempDir::new().unwrap();
    write_config(&dir, r#"api_key = "sk-from-file""#);
    let config_path = dir.path().join("deepseek-tray").join("config.toml");

    let mut config = Config::load_from_path(&config_path).unwrap();
    let key = config.resolve_api_key(&config_path, None).unwrap();
    assert_eq!(key, "sk-from-file");
}

#[test]
fn test_resolve_api_key_missing_returns_error() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("deepseek-tray").join("config.toml");

    let mut config = Config::default();
    let result = config.resolve_api_key(&config_path, None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("API key"));
}

#[test]
fn test_resolve_api_key_empty_env_falls_through() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let dir = TempDir::new().unwrap();
    write_config(&dir, r#"api_key = "sk-from-file""#);
    let config_path = dir.path().join("deepseek-tray").join("config.toml");

    // Empty env var should be ignored, fall through to config file
    env::set_var("DEEPSEEK_API_KEY", "");
    let mut config = Config::load_from_path(&config_path).unwrap();
    let key = config.resolve_api_key(&config_path, None).unwrap();
    assert_eq!(key, "sk-from-file");
    env::remove_var("DEEPSEEK_API_KEY");
}

#[test]
fn test_resolve_api_key_empty_config_falls_through_to_dialog() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let dir = TempDir::new().unwrap();
    write_config(&dir, r#"api_key = """#);
    let config_path = dir.path().join("deepseek-tray").join("config.toml");

    // Empty config key + no env var → should fall through to dialog
    let mut config = Config::load_from_path(&config_path).unwrap();
    let key = config.resolve_api_key(&config_path, Some(|_, _, _| {
        Some("sk-from-dialog".into())
    })).unwrap();
    assert_eq!(key, "sk-from-dialog");
    // Key should have been saved to config
    assert_eq!(config.api_key.as_deref(), Some("sk-from-dialog"));
}

#[test]
fn test_resolve_api_key_from_dialog_saves_to_config() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("deepseek-tray").join("config.toml");

    let mut config = Config::default();
    let key = config.resolve_api_key(&config_path, Some(|_, _, _| {
        Some("  sk-dialog-trimmed  ".into())
    })).unwrap();
    assert_eq!(key, "sk-dialog-trimmed");
    assert_eq!(config.api_key.as_deref(), Some("sk-dialog-trimmed"));
    // Verify it was actually saved to disk
    let reloaded = Config::load_from_path(&config_path).unwrap();
    assert_eq!(reloaded.api_key.as_deref(), Some("sk-dialog-trimmed"));
}

#[test]
fn test_resolve_api_key_empty_dialog_returns_error() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("deepseek-tray").join("config.toml");

    let mut config = Config::default();
    let result = config.resolve_api_key(&config_path, Some(|_, _, _| {
        Some("".into())  // empty input from dialog
    }));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("API key"));
}
