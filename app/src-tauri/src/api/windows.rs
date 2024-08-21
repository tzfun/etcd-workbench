use std::path::PathBuf;
use std::process::Command;

use tauri::utils::config::WindowConfig;
use tauri::{Manager, WindowBuilder, WindowUrl};

use crate::error::LogicError;

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
        let main = WindowBuilder::from_config(app_handle, config)
            .build()
            .unwrap();
        main.show().unwrap();
        #[cfg(target_os = "windows")]
        window_shadows::set_shadow(&main, true).unwrap();
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
            resizable: false,
            center: true,
            url: WindowUrl::App(PathBuf::from("/?page=setting")),
            decorations: false,
            transparent: true,
            ..<_>::default()
        };
        let setting = WindowBuilder::from_config(&app_handle, config)
            .build()
            .unwrap();
        setting.show().unwrap();
        #[cfg(target_os = "windows")]
        window_shadows::set_shadow(&setting, true).unwrap();
    }
}

#[tauri::command]
pub fn exit_app() {
    std::process::exit(0);
}

#[tauri::command]
pub fn open_folder(path: String, select_file: Option<String>) -> Result<(), LogicError> {
    let mut path_buf = PathBuf::new();
    path_buf.push(path);

    if let Some(file) = select_file {
        path_buf.push(file);
    }

    if !path_buf.exists() {
        return Err(LogicError::ResourceNotExist("File does not exist"));
    }

    let full_path = path_buf.to_str().unwrap();

    #[cfg(windows)]
    {
        Command::new("explorer")
            .args(["/select,", full_path])
            .spawn()
            .unwrap();
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .args(["-R", full_path])
            .spawn()
            .unwrap();
    }
    Ok(())
}
