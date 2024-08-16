use std::path::PathBuf;

use tauri::{Manager, SystemTrayEvent, WindowBuilder, WindowUrl};
use tauri::utils::config::WindowConfig;
use window_shadows::set_shadow;

pub fn tray_menu_handle(app_handle: &tauri::AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::MenuItemClick { id, .. } => {
            match id.as_str() {
                "quit" => {
                    std::process::exit(0)
                },
                "hide" => {
                    let item_handle = app_handle.tray_handle().get_item(&id);

                    if let Some(window) = app_handle.get_window("main") {
                        if window.is_visible().unwrap() {
                            window.hide().unwrap();
                            item_handle.set_title("Show Workbench").unwrap();
                        } else {
                            window.show().unwrap();
                            item_handle.set_title("Hide Workbench").unwrap();
                        }
                    }
                },
                _=>{}
            }
        }
        SystemTrayEvent::LeftClick { .. } => {
            open_main_window0(app_handle)
        }
        SystemTrayEvent::RightClick { .. } => {}
        SystemTrayEvent::DoubleClick { .. } => {},
        _ => {}
    }
}

pub fn open_main_window0(app_handle: &tauri::AppHandle) {
    // 关闭初始屏幕
    if let Some(splashscreen) = app_handle.get_window("splashscreen") {
        splashscreen.close().unwrap();
    }
    // 显示主窗口
    if let Some(main) = app_handle.get_window("main") {
        main.show().unwrap();
        main.set_focus().unwrap();
    } else {
        let config = WindowConfig {
            label: String::from("main"),
            title: String::from("Etcd Workbench"),
            width: 1400f64,
            height: 1000f64,
            center: true,
            decorations: false,
            transparent: true,
            closable: false,
            ..<_>::default()
        };
        let main = WindowBuilder::from_config(app_handle, config).build().unwrap();
        main.show().unwrap();
        set_shadow(&main, true).unwrap();
        main.set_focus().unwrap();
    }
}

#[tauri::command]
pub fn open_main_window(app_handle: tauri::AppHandle) {
    open_main_window0(&app_handle)
}

#[tauri::command]
pub async fn open_setting_window(app_handle: tauri::AppHandle, window: tauri::Window) {
    if let Some(setting) = window.get_window("setting") {
        if setting.is_visible().unwrap() {
            setting.set_focus().unwrap()
        } else {
            setting.show().unwrap();
        }
    } else {
        let config = WindowConfig {
            label: String::from("setting"),
            title: String::from("Settings"),
            width: 1200f64,
            height: 800f64,
            center: true,
            url: WindowUrl::App(PathBuf::from("/?page=setting")),
            decorations: false,
            transparent: true,
            ..<_>::default()
        };
        let setting = WindowBuilder::from_config(&app_handle, config).build().unwrap();
        setting.show().unwrap();
        set_shadow(&setting, true).unwrap();
    }
}

#[tauri::command]
pub async fn close_all_window(window: tauri::Window) {
    for (_, window) in window.windows() {
        window.close().unwrap()
    }
}