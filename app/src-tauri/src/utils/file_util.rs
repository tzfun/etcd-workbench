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
pub static CONN_CONFIG_DIR: &'static str = "connections";
pub static DATA_DIR: &'static str = "data";
pub static SETTINGS_FILE: &'static str = "settings";
pub static META_FILE: &'static str = "meta";

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
    let path = get_storage_root_path();
    info!("initialized local path: {}", path.to_str().unwrap_or(""));
    if !path.exists() {
        fs::create_dir_all(path)?;
    }

    let config_path = get_conn_config_dir_path();
    if !config_path.exists() {
        fs::create_dir_all(&config_path)?;
    }

    Ok(())
}

/// 获取连接配置目录路径
pub fn get_conn_config_dir_path() -> PathBuf {
    let mut path = get_data_path();
    path.push(CONN_CONFIG_DIR);
    path
}

/// 获取设置文件路径
pub fn get_setting_file_path() -> PathBuf {
    let mut path = get_data_path();
    path.push(SETTINGS_FILE);
    path
}

/// 存储数据的目录，存放配置、元数据、设置等
pub fn get_data_path() -> PathBuf {
    let mut path = get_storage_root_path();
    path.push(DATA_DIR);
    path
}

/// 本地存储根目录
fn get_storage_root_path() -> PathBuf {
    let mut path = local_data_dir().unwrap();
    path.push(BASE_DIR);
    path
}