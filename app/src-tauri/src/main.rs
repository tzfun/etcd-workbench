// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::{LevelFilter, warn};
use tauri::Manager;
use window_shadows::set_shadow;

use crate::utils::file_util;

mod api;
mod transport;
mod etcd;
mod ssh;
mod error;
mod utils;

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Debug)
        .init();

    file_util::init().unwrap();

    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_window("main").unwrap();
            if let Err(e) = set_shadow(&window, true) {
                warn!("Can not set window shadow: {}", e)
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            api::connection::connect_test,
            api::connection::connect,
            api::connection::disconnect,
            api::settings::get_settings,
            api::settings::save_connection,
            api::settings::remove_connection,
            api::settings::get_connection_list,
            api::kv::kv_get_all_keys,
            api::kv::kv_get,
            api::kv::kv_get_by_version,
            api::kv::kv_put,
            api::kv::kv_put_with_lease,
            api::kv::kv_delete,
            api::kv::kv_get_history_versions,
            api::maintenance::get_cluster,
            api::lease::leases,
            api::lease::lease_get,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
