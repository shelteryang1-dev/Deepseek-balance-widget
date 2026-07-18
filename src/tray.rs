use anyhow::{Context, Result};
use muda::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tray_icon::{TrayIcon, TrayIconBuilder};

use crate::render::{render_balance_icon, render_status_icon, RenderedIcon};

#[derive(Debug, Clone)]
pub enum TrayState {
    Balance(String),
    NoKey,
    NetworkError,
    Unauthorized,
    RateLimited,
}

impl TrayState {
    pub fn icon_text(&self) -> &str {
        match self {
            TrayState::Balance(_) => "",
            TrayState::NoKey => "--",
            TrayState::NetworkError => "ERR",
            TrayState::Unauthorized => "KEY",
            TrayState::RateLimited => "429",
        }
    }

    pub fn tooltip(&self) -> String {
        match self {
            TrayState::Balance(text) => format!("DeepSeek 余额: {}", text),
            TrayState::NoKey => "未配置 API Key，请右键设置".into(),
            TrayState::NetworkError => "网络连接失败，点击刷新重试".into(),
            TrayState::Unauthorized => "API Key 无效，请右键重新配置".into(),
            TrayState::RateLimited => "请求过于频繁，稍后重试".into(),
        }
    }
}

pub mod menu_id {
    pub const REFRESH: &str = "refresh";
    pub const COPY: &str = "copy";
    pub const INTERVAL_15: &str = "interval_15";
    pub const INTERVAL_30: &str = "interval_30";
    pub const INTERVAL_60: &str = "interval_60";
    pub const SET_API_KEY: &str = "set_api_key";
    pub const TOGGLE_AUTOSTART: &str = "toggle_autostart";
    pub const QUIT: &str = "quit";
}

pub struct MenuItems {
    pub interval_15: CheckMenuItem,
    pub interval_30: CheckMenuItem,
    pub interval_60: CheckMenuItem,
    pub auto_start: CheckMenuItem,
}

pub fn build_menu() -> Result<(Menu, MenuItems)> {
    let menu = Menu::new();

    let refresh = MenuItem::with_id(menu_id::REFRESH, "刷新余额", true, None);
    let copy = MenuItem::with_id(menu_id::COPY, "复制余额", true, None);
    let sep1 = PredefinedMenuItem::separator();

    let interval_sub = Submenu::with_id("interval_sub", "刷新间隔", true);
    let interval_15 =
        CheckMenuItem::with_id(menu_id::INTERVAL_15, "15 分钟", true, false, None);
    let interval_30 =
        CheckMenuItem::with_id(menu_id::INTERVAL_30, "30 分钟", true, true, None);
    let interval_60 =
        CheckMenuItem::with_id(menu_id::INTERVAL_60, "60 分钟", true, false, None);
    interval_sub
        .append(&interval_15)
        .and_then(|_| interval_sub.append(&interval_30))
        .and_then(|_| interval_sub.append(&interval_60))
        .context("failed to build interval submenu")?;

    let sep2 = PredefinedMenuItem::separator();
    let set_key = MenuItem::with_id(menu_id::SET_API_KEY, "设置 API Key", true, None);
    let auto_start =
        CheckMenuItem::with_id(menu_id::TOGGLE_AUTOSTART, "开机自启", true, false, None);
    let sep3 = PredefinedMenuItem::separator();
    let quit = MenuItem::with_id(menu_id::QUIT, "退出", true, None);

    menu.append(&refresh)
        .and_then(|_| menu.append(&copy))
        .and_then(|_| menu.append(&sep1))
        .and_then(|_| menu.append(&interval_sub))
        .and_then(|_| menu.append(&sep2))
        .and_then(|_| menu.append(&set_key))
        .and_then(|_| menu.append(&auto_start))
        .and_then(|_| menu.append(&sep3))
        .and_then(|_| menu.append(&quit))
        .context("failed to build menu")?;

    let items = MenuItems { interval_15, interval_30, interval_60, auto_start };

    Ok((menu, items))
}

pub fn to_tray_icon(rendered: &RenderedIcon) -> tray_icon::Icon {
    tray_icon::Icon::from_rgba(rendered.rgba.clone(), rendered.width, rendered.height)
        .expect("invalid icon dimensions")
}

pub fn render_icon(state: &TrayState) -> Result<RenderedIcon> {
    match state {
        TrayState::Balance(text) => render_balance_icon(text),
        _ => render_status_icon(state.icon_text()),
    }
}

pub fn build_tray(menu: Menu) -> Result<TrayIcon> {
    let placeholder = render_status_icon("--")?;
    let icon = to_tray_icon(&placeholder);

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("DeepSeek Tray")
        .with_icon(icon)
        .build()
        .context("failed to create tray icon")?;

    Ok(tray)
}

pub fn update_interval_checks(items: &MenuItems, current: u32) {
    items.interval_15.set_checked(current == 15);
    items.interval_30.set_checked(current == 30);
    items.interval_60.set_checked(current == 60);
}
