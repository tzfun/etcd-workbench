use std::path::PathBuf;

use tauri::{Manager, WindowBuilder, WindowUrl};
use tauri::utils::config::WindowConfig;
use window_shadows::set_shadow;

#[tauri::command]
pub async fn open_main_window(window: tauri::Window) {
    // 关闭初始屏幕
    if let Some(splashscreen) = window.get_window("splashscreen") {
        splashscreen.close().unwrap();
    }
    // 显示主窗口
    window.get_window("main").unwrap().show().unwrap();
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
            width: 1200f64,
            height: 800f64,
            center: true,
            url: WindowUrl::App(PathBuf::from("/?page=setting")),
            decorations: false,
            transparent: true,
            ..<_>::default()
        };
        let setting = WindowBuilder::from_config(&app_handle, config).build().unwrap();
        set_shadow(&setting, true).unwrap();
        setting.show().unwrap();
    }
}