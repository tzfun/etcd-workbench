// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use api::settings::get_global_store_from_file;
use log::{debug, info, LevelFilter};
use tauri::{Manager, PhysicalSize, RunEvent, Size, WindowEvent};

// use crate::api::windows::tray_menu_handle;
use crate::utils::file_util;

mod api;
mod transport;
mod etcd;
mod ssh;
mod error;
mod utils;
mod proto;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    let mut log_level = LevelFilter::Info;
    if api::settings::is_debug_model() {
        log_level = LevelFilter::Debug;
        println!("Running in debug model...");
    } else {
        println!("Running in release model...");
    }
    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .filter_module("tao::platform_impl::platform::event_loop::runner", LevelFilter::Error)
        .init();
    info!("env logger initialized");

    file_util::init().unwrap();
    info!("file util initialized");

    tauri::Builder::default()
        .setup(|app| {
            debug!("loading window size from user setting file");
            let store = get_global_store_from_file().unwrap();
            debug!("loading global store success");

            let window_init_state = store.window_init_state;
            for (name, window) in app.windows() {
                #[cfg(target_os = "windows")]
                if name.ne("splashscreen") {
                    log::debug!("set window shadow: {}", name);
                    window_shadows::set_shadow(&window, true).unwrap();
                }
                if name.eq("main") {
                    debug!("try to set physical size of main window");
                    if let Some(state) = &window_init_state {
                        if state.main_window_fullscreen {
                            debug!("The window returns to fullscreen mode");
                            window.set_fullscreen(true).unwrap();
                        } else if state.main_window_maximize {
                            debug!("The window returns to maximizable mode");
                            window.maximize().unwrap();
                        } else if state.main_window_width > 0f64 && state.main_window_height > 0f64 {
                            debug!("Window restored to size: {} x {}", state.main_window_width, state.main_window_height);
                            window.set_size(Size::from(PhysicalSize {
                                width: state.main_window_width,
                                height: state.main_window_height,
                            })).unwrap();
                        }
                        debug!("main window size initialized");
                    } else {
                        debug!("no setting of physical size");
                    }
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            api::windows::client_error,
            api::windows::open_main_window,
            api::windows::open_setting_window,
            api::windows::exit_app,
            api::windows::open_folder,
            api::windows::get_download_path,
            api::connection::connect_test,
            api::connection::connect,
            api::connection::disconnect,
            api::connection::save_connection,
            api::connection::remove_connection,
            api::connection::get_connection_list,
            api::connection::export_connection,
            api::connection::import_connection,
            api::connection::update_key_collection,
            api::connection::set_key_monitor,
            api::connection::remove_key_monitor,
            api::settings::get_settings,
            api::settings::get_global_store,
            api::settings::save_settings,
            api::settings::save_global_store,
            api::settings::get_app_version,
            api::settings::is_debug_model,
            api::kv::kv_get_all_keys,
            api::kv::kv_get_all_keys_paging,
            api::kv::kv_get,
            api::kv::kv_get_by_version,
            api::kv::kv_put,
            api::kv::kv_put_with_lease,
            api::kv::kv_delete,
            api::kv::kv_get_history_versions,
            api::maintenance::get_cluster,
            api::maintenance::maintenance_defragment,
            api::maintenance::maintenance_create_snapshot_task,
            api::maintenance::maintenance_stop_snapshot_task,
            api::maintenance::maintenance_remove_snapshot_task,
            api::maintenance::maintenance_list_snapshot_task,
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
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app, event| {
            match event {
                RunEvent::Exit => {}
                RunEvent::ExitRequested { .. } => {}
                RunEvent::WindowEvent {
                    label,
                    event: win_event,
                    ..
                } => {
                    match win_event {
                        WindowEvent::Resized(_) => {}
                        WindowEvent::Moved(_) => {}
                        WindowEvent::CloseRequested { api, .. } => {
                            if label.eq("main") {
                                app.emit_all("confirm_exit", ()).unwrap();
                                api.prevent_close();
                            } else if label.eq("splashscreen") {

                            } else {
                                let win = app.get_window(label.as_str()).unwrap();
                                win.hide().unwrap();
                                api.prevent_close();
                            }
                        }
                        WindowEvent::Destroyed => {}
                        WindowEvent::Focused(_) => {}
                        WindowEvent::ScaleFactorChanged { .. } => {}
                        WindowEvent::FileDrop(_) => {}
                        WindowEvent::ThemeChanged(_) => {},
                        _=>{}
                    }
                }
                RunEvent::Ready => {}
                RunEvent::Resumed => {}
                RunEvent::MainEventsCleared => {}
                _=>{}
            }
        });
}
