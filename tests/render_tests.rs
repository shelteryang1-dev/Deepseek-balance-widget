use deepseek_tray::render::{render_balance_icon, render_status_icon};

#[test]
fn test_render_balance_icon_valid() {
    let icon = render_balance_icon("¥21").unwrap();
    assert!(icon.width >= 16);
    assert!(icon.height >= 16);
    assert_eq!(icon.rgba.len(), (icon.width * icon.height * 4) as usize);
    // GDI-rendered text should have visible pixels
    let visible = icon.rgba.chunks(4).filter(|px| px[3] > 0).count();
    assert!(visible > 0, "GDI icon should have visible pixels");
}

#[test]
fn test_render_status_icon_valid() {
    let icon = render_status_icon("ERR").unwrap();
    assert!(icon.width >= 16);
    assert!(icon.height >= 16);
    let visible = icon.rgba.chunks(4).filter(|px| px[3] > 0).count();
    assert!(visible > 0, "status icon should have visible pixels");
}

#[test]
fn test_different_statuses_differ() {
    let err = render_status_icon("ERR").unwrap();
    let gray = render_status_icon("--").unwrap();
    assert_ne!(err.rgba, gray.rgba);
}

#[test]
fn test_balance_vs_status_differ() {
    let bal = render_balance_icon("¥21").unwrap();
    let err = render_status_icon("ERR").unwrap();
    assert_ne!(bal.rgba, err.rgba);
}
