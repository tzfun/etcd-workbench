// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::{debug, LevelFilter};
use tauri::{Manager, WindowEvent};
// use crate::api::windows::tray_menu_handle;
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
            for (name, window) in app.windows() {
                if name.ne("splashscreen") {
                    debug!("set window shadow: {}", name);
                    #[cfg(target_os = "windows")]
                    window_shadows::set_shadow(&window, true).unwrap()
                }
                if name.eq("main") {
                    window.on_window_event(|e| {
                        match e {
                            WindowEvent::Resized(_) => {}
                            WindowEvent::Moved(_) => {}
                            WindowEvent::CloseRequested { .. } => {
                                std::process::exit(0);
                            }
                            WindowEvent::Destroyed => {}
                            WindowEvent::Focused(_) => {}
                            WindowEvent::ScaleFactorChanged { .. } => {}
                            WindowEvent::FileDrop(_) => {}
                            WindowEvent::ThemeChanged(_) => {},
                            _=>{}
                        }
                    })
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            api::windows::open_main_window,
            api::windows::open_setting_window,
            api::windows::exit_app,
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
            api::maintenance::maintenance_defragment,
            api::lease::leases,
            api::lease::lease_get,
            api::lease::lease_grant,
            api::lease::lease_revoke,
            api::user::user_list,
            api::user::user_add,
            api::user::user_delete,
            api::user::user_change_password,
            api::user::user_grant_role,
            api::user::user_revoke_role,
            api::user::auth_enable,
            api::user::auth_disable,
            api::role::role_list,
            api::role::role_add,
            api::role::role_delete,
            api::role::role_get_permissions,
            api::role::role_grant_permission,
            api::role::role_revoke_permission,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
