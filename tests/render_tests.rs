use deepseek_tray::render::{render_balance_icon, render_status_icon};

#[test]
fn test_render_balance_icon_returns_valid_icon() {
    let icon = render_balance_icon("88.50").unwrap();
    assert!(icon.width > 0);
    assert!(icon.height > 0);
    assert!(icon.rgba.len() > 0);
    assert_eq!(icon.rgba.len(), (icon.width * icon.height * 4) as usize);
}

#[test]
fn test_render_status_icons_different() {
    let a = render_status_icon("ERR").unwrap();
    let b = render_status_icon("OK").unwrap();
    assert_ne!(a.rgba, b.rgba);
}

#[test]
fn test_render_different_texts_different() {
    // Strings of different lengths must produce different pixel data
    let icon1 = render_balance_icon("A").unwrap();
    let icon2 = render_balance_icon("BBBBB").unwrap();
    assert_ne!(icon1.rgba, icon2.rgba);
}

#[test]
fn test_render_produces_non_transparent_pixels() {
    let icon = render_balance_icon("88.50").unwrap();
    let has_visible = icon.rgba.chunks(4).any(|px| px[3] > 0);
    assert!(has_visible, "icon should have visible pixels");
}
