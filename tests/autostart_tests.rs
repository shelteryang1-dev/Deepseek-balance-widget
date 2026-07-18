use deepseek_tray::autostart;

#[test]
fn test_is_enabled_returns_bool() {
    let result = autostart::is_enabled();
    assert!(result == true || result == false);
}

#[test]
fn test_toggle_sequence() {
    // Save original state
    let was_enabled = autostart::is_enabled();

    // Enable
    autostart::enable().unwrap();
    assert!(autostart::is_enabled(), "should be enabled after enable()");

    // Disable
    autostart::disable().unwrap();
    assert!(!autostart::is_enabled(), "should be disabled after disable()");

    // Restore original state
    if was_enabled {
        autostart::enable().unwrap();
    }
}
