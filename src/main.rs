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

#[derive(Debug, Clone)]
enum UserEvent {
    UpdateState(TrayState),
    UpdateMenuChecks(u32, bool),
    RefreshRequested,
}

fn main() {
    unsafe {
        let _ = windows::Win32::UI::HiDpi::SetProcessDpiAwareness(
            windows::Win32::UI::HiDpi::PROCESS_PER_MONITOR_DPI_AWARE,
        );
    }

    env_logger::init();
    log::info!("DeepSeek Tray starting...");

    let config_path = config::Config::config_path();
    let mut cfg = config::Config::load().expect("failed to load config");

    let api_key = cfg.resolve_api_key(&config_path, None).ok();

    if cfg.auto_start {
        let _ = autostart::enable();
    }

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let (menu, menu_items) = tray::build_menu().expect("failed to build menu");
    let tray_icon = tray::build_tray(menu).expect("failed to create tray icon");

    if autostart::is_enabled() {
        menu_items.auto_start.set_checked(true);
        cfg.auto_start = true;
        let _ = cfg.save();
    }

    tray::update_interval_checks(&menu_items, cfg.refresh_interval_minutes);

    let current_state = if api_key.is_some() {
        TrayState::NetworkError
    } else {
        TrayState::NoKey
    };

    {
        let icon = tray::render_icon(&current_state).expect("failed to render icon");
        tray_icon
            .set_icon(Some(tray::to_tray_icon(&icon)))
            .expect("failed to set icon");
        tray_icon
            .set_tooltip(Some(&current_state.tooltip()))
            .expect("failed to set tooltip");
    }

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
            if let Some(ref key) = api_key_clone {
                let state = fetch_and_map(key).await;
                let _ = proxy_clone.send_event(UserEvent::UpdateState(state));
            }

            let mut tick =
                tokio::time::interval(std::time::Duration::from_secs(interval as u64 * 60));
            tick.tick().await;

            while running_clone.load(Ordering::Relaxed) {
                tick.tick().await;
                if let Some(ref key) = api_key_clone {
                    let state = fetch_and_map(key).await;
                    let _ = proxy_clone.send_event(UserEvent::UpdateState(state));
                }
            }
        });
    });

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

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
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Builder::new_current_thread()
                            .enable_time()
                            .build()
                            .unwrap();
                        rt.block_on(async {
                            if let Ok(mut cfg) = config::Config::load() {
                                if let Ok(key) =
                                    cfg.resolve_api_key(&config::Config::config_path(), None)
                                {
                                    let state = fetch_and_map(&key).await;
                                    let _ = p.send_event(UserEvent::UpdateState(state));
                                }
                            }
                        });
                    });
                }
            }
        }

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
                                if let Ok(balance) = api::fetch_balance(key).await {
                                    let text = format!("{:.0}", balance.total);
                                    if let Ok(mut cb) = arboard::Clipboard::new() {
                                        let _ = cb.set_text(&text);
                                        log::info!("copied to clipboard: {}", text);
                                    }
                                }
                            }
                        });
                    });
                }
                menu_id::INTERVAL_15 => {
                    if let Ok(mut cfg) = config::Config::load() {
                        cfg.refresh_interval_minutes = 15;
                        let _ = cfg.save();
                        let _ =
                            proxy.send_event(UserEvent::UpdateMenuChecks(15, cfg.auto_start));
                    }
                }
                menu_id::INTERVAL_30 => {
                    if let Ok(mut cfg) = config::Config::load() {
                        cfg.refresh_interval_minutes = 30;
                        let _ = cfg.save();
                        let _ =
                            proxy.send_event(UserEvent::UpdateMenuChecks(30, cfg.auto_start));
                    }
                }
                menu_id::INTERVAL_60 => {
                    if let Ok(mut cfg) = config::Config::load() {
                        cfg.refresh_interval_minutes = 60;
                        let _ = cfg.save();
                        let _ =
                            proxy.send_event(UserEvent::UpdateMenuChecks(60, cfg.auto_start));
                    }
                }
                menu_id::SET_API_KEY => {
                    let config_path = config::Config::config_path();
                    if let Some(parent) = config_path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let _ = std::fs::write(
                        &config_path,
                        "# DeepSeek Tray 配置文件\n\
                         # 把下面的 \"replace-with-your-api-key\" 替换为你的 API Key\n\
                         # 保存后右键托盘图标 → 刷新余额\n\
                         \n\
                         api_key = \"replace-with-your-api-key\"\n\
                         refresh_interval_minutes = 30\n\
                         auto_start = false\n",
                    );
                    let _ = std::process::Command::new("notepad")
                        .arg(&config_path)
                        .spawn();
                }
                menu_id::TOGGLE_AUTOSTART => {
                    let enabled = autostart::is_enabled();
                    let new_state = !enabled;
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

async fn fetch_and_map(api_key: &str) -> TrayState {
    match api::fetch_balance(api_key).await {
        Ok(balance) => {
            let text = format!("{:.0}", balance.total);
            log::info!(
                "balance: {} (topped_up={:.2}, granted={:.2})",
                text,
                balance.topped_up,
                balance.granted
            );
            TrayState::Balance(text)
        }
        Err(api::ApiError::Unauthorized) => {
            log::warn!("API key invalid");
            TrayState::Unauthorized
        }
        Err(api::ApiError::RateLimited) => {
            log::warn!("rate limited");
            TrayState::RateLimited
        }
        Err(e) => {
            log::error!("API error: {}", e);
            TrayState::NetworkError
        }
    }
}
