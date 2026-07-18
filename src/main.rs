#![windows_subsystem = "windows"]

mod api;
mod autostart;
mod config;
mod render;
mod tray;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    log::info!("DeepSeek Tray starting...");
    todo!("main event loop")
}
