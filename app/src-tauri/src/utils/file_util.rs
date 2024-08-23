#![allow(unused)]
use std::{fs, io};
use std::env::temp_dir;
use std::fs::File;
use std::io::Write;
use std::path::{PathBuf};
use log::info;
use tauri::api::path::{BaseDirectory, local_data_dir};
use uuid::Uuid;

static BASE_DIR: &'static str = "Etcd Workbench";
pub static CONFIG_DIR: &'static str = "configuration";
pub static SETTINGS_FILE: &'static str = "settings";

/// 创建一个临时文件，并返回该文件的全路径
pub fn create_temp_file(data: &[u8]) -> io::Result<String> {
    let mut dir = temp_dir();
    let file_name = format!("{}", Uuid::new_v4());
    dir.push(file_name);
    let file_full_name = dir.display().to_string();
    let mut file = File::create(dir)?;
    file.write(data)?;
    Ok(file_full_name)
}

pub fn init() -> io::Result<()> {
    let path = get_storage_path();
    info!("initialized local path: {}", path.to_str().unwrap_or(""));
    if !path.exists() {
        fs::create_dir_all(path)?;
    }

    let config_path = get_config_dir_path();
    if !config_path.exists() {
        fs::create_dir_all(&config_path)?;
    }

    Ok(())
}

pub fn get_config_dir_path() -> PathBuf {
    let mut path = get_storage_path();
    path.push(CONFIG_DIR);
    path
}

pub fn get_setting_file_path() -> PathBuf {
    let mut path = get_storage_path();
    path.push(SETTINGS_FILE);
    path
}

pub fn get_storage_path() -> PathBuf {
    let mut path = local_data_dir().unwrap();
    path.push(BASE_DIR);
    path
}