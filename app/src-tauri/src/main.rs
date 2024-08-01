// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod transport;
mod etcd;

use log::{LevelFilter, warn};
use tauri::Manager;
use window_shadows::set_shadow;

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Debug)
        .init();

    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_window("main").unwrap();
            if let Err(e) = set_shadow(&window, true) {
                warn!("Can not set window shadow: {}", e)
            }

            #[cfg(target_os = "macos")]
            window.set_decorations(true)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            api::connection::connect_test,
            api::connection::connect,
            api::connection::disconnect,
            api::kv::kv_get_all_keys
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
