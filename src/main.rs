#![windows_subsystem = "windows"]

mod api;
mod autostart;
mod config;
mod render;
mod tray;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray::{menu_id, TrayState};

/// Events sent from background tasks to the main thread via the event loop.
#[derive(Debug, Clone)]
enum UserEvent {
    UpdateState(TrayState),
    /// Update menu check marks: (interval_minutes, auto_start_enabled)
    UpdateMenuChecks(u32, bool),
    RefreshRequested,
    Quit,
}

fn main() {
    env_logger::init();
    log::info!("DeepSeek Tray starting...");

    // ── Load config ──
    let config_path = config::Config::config_path();
    let mut cfg = config::Config::load().expect("加载配置失败");

    // Resolve API key (with dialog fallback)
    let api_key = cfg
        .resolve_api_key(&config_path, Some(tinyfiledialogs::input_box))
        .ok();

    // Apply autostart setting
    if cfg.auto_start {
        let _ = autostart::enable();
    }

    // ── Event loop ──
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event()
        .build();
    let proxy = event_loop.create_proxy();

    // ── Build tray ──
    let (menu, menu_items) = tray::build_menu().expect("构建菜单失败");
    let tray_icon = tray::build_tray(menu).expect("创建托盘图标失败");

    // Sync autostart check state
    if autostart::is_enabled() {
        menu_items.auto_start.set_checked(true);
        cfg.auto_start = true;
        let _ = cfg.save();
    }

    tray::update_interval_checks(&menu_items, cfg.refresh_interval_minutes);

    // ── Initial state ──
    let current_state = if api_key.is_some() {
        TrayState::NetworkError // will refresh immediately
    } else {
        TrayState::NoKey
    };

    {
        let icon = tray::render_icon(&current_state).expect("渲染初始图标失败");
        tray_icon
            .set_icon(Some(tray::to_tray_icon(&icon)))
            .expect("设置托盘图标失败");
        tray_icon
            .set_tooltip(Some(&current_state.tooltip()))
            .expect("设置托盘提示失败");
    }

    // ── Background thread: tokio runtime for HTTP + timers ──
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    let proxy_clone = proxy.clone();
    let interval = cfg.refresh_interval_minutes;
    let api_key_clone = api_key.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime");

        rt.block_on(async move {
            // Initial fetch
            if let Some(ref key) = api_key_clone {
                let state = fetch_and_map(key).await;
                let _ = proxy_clone.send_event(UserEvent::UpdateState(state));
            }

            // Periodic refresh
            let mut tick =
                tokio::time::interval(std::time::Duration::from_secs(interval as u64 * 60));
            tick.tick().await; // skip first immediate tick

            while running_clone.load(Ordering::Relaxed) {
                tick.tick().await;
                if let Some(ref key) = api_key_clone {
                    let state = fetch_and_map(key).await;
                    let _ = proxy_clone.send_event(UserEvent::UpdateState(state));
                }
            }
        });
    });

    // ── Main event loop ──
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Handle our custom user events (from background thread)
        if let tao::event::Event::UserEvent(ue) = event {
            match ue {
                UserEvent::UpdateState(state) => {
                    if let Ok(icon) = tray::render_icon(&state) {
                        let tooltip = state.tooltip();
                        let _ = tray_icon.set_icon(Some(tray::to_tray_icon(&icon)));
                        let _ = tray_icon.set_tooltip(Some(&tooltip));
                    }
                }
                UserEvent::UpdateMenuChecks(interval, auto_start) => {
                    tray::update_interval_checks(&menu_items, interval);
                    menu_items.auto_start.set_checked(auto_start);
                }
                UserEvent::RefreshRequested => {
                    let p = proxy.clone();
                    let key = api_key.clone();
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Builder::new_current_thread()
                            .enable_time()
                            .build()
                            .unwrap();
                        rt.block_on(async {
                            if let Some(ref key) = key {
                                let state = fetch_and_map(key).await;
                                let _ = p.send_event(UserEvent::UpdateState(state));
                            }
                        });
                    });
                }
                UserEvent::Quit => {
                    *control_flow = ControlFlow::Exit;
                }
            }
        }

        // Handle menu events (muda sends via channel)
        while let Ok(event) = muda::MenuEvent::receiver().try_recv() {
            match event.id.as_ref() {
                menu_id::REFRESH => {
                    let _ = proxy.send_event(UserEvent::RefreshRequested);
                }
                menu_id::COPY => {
                    let key = api_key.clone();
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Builder::new_current_thread()
                            .enable_time()
                            .build()
                            .unwrap();
                        rt.block_on(async {
                            if let Some(ref key) = key {
                                match api::fetch_balance(key).await {
                                    Ok(balance) => {
                                        let text = format!("¥{:.2}", balance.total);
                                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                            let _ = clipboard.set_text(&text);
                                            log::info!("已复制到剪贴板: {}", text);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        });
                    });
                }
                menu_id::INTERVAL_15 => {
                    if let Ok(mut cfg) = config::Config::load() {
                        cfg.refresh_interval_minutes = 15;
                        let _ = cfg.save();
                        let _ = proxy
                            .send_event(UserEvent::UpdateMenuChecks(15, cfg.auto_start));
                    }
                }
                menu_id::INTERVAL_30 => {
                    if let Ok(mut cfg) = config::Config::load() {
                        cfg.refresh_interval_minutes = 30;
                        let _ = cfg.save();
                        let _ = proxy
                            .send_event(UserEvent::UpdateMenuChecks(30, cfg.auto_start));
                    }
                }
                menu_id::INTERVAL_60 => {
                    if let Ok(mut cfg) = config::Config::load() {
                        cfg.refresh_interval_minutes = 60;
                        let _ = cfg.save();
                        let _ = proxy
                            .send_event(UserEvent::UpdateMenuChecks(60, cfg.auto_start));
                    }
                }
                menu_id::SET_API_KEY => {
                    if let Some(key) = tinyfiledialogs::input_box(
                        "设置 API Key",
                        "请输入 DeepSeek API Key:",
                        "",
                    ) {
                        let key = key.trim().to_string();
                        if !key.is_empty() {
                            if let Ok(mut cfg) = config::Config::load() {
                                cfg.api_key = Some(key);
                                let _ = cfg.save();
                                let _ = proxy.send_event(UserEvent::RefreshRequested);
                            }
                        }
                    }
                }
                menu_id::TOGGLE_AUTOSTART => {
                    let is_enabled = autostart::is_enabled();
                    let new_state = !is_enabled;
                    let _ = autostart::set_enabled(new_state);
                    if let Ok(mut cfg) = config::Config::load() {
                        cfg.auto_start = new_state;
                        let _ = cfg.save();
                        let _ = proxy.send_event(UserEvent::UpdateMenuChecks(
                            cfg.refresh_interval_minutes,
                            new_state,
                        ));
                    }
                }
                menu_id::QUIT => {
                    running.store(false, Ordering::Relaxed);
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            }
        }
    });
}

/// Fetch balance from API and map to TrayState.
async fn fetch_and_map(api_key: &str) -> TrayState {
    match api::fetch_balance(api_key).await {
        Ok(balance) => {
            let text = format!("¥{:.2}", balance.total);
            log::info!(
                "余额: {} (充值¥{:.2} + 赠送¥{:.2})",
                text,
                balance.topped_up,
                balance.granted
            );
            TrayState::Balance(text)
        }
        Err(api::ApiError::Unauthorized) => {
            log::warn!("API Key 无效");
            TrayState::Unauthorized
        }
        Err(api::ApiError::RateLimited) => {
            log::warn!("API 请求限流");
            TrayState::RateLimited
        }
        Err(e) => {
            log::error!("API 请求失败: {}", e);
            TrayState::NetworkError
        }
    }
}
