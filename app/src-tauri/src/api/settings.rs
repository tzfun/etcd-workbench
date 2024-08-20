use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use lazy_static::lazy_static;
use log::debug;
use tauri::AppHandle;
use tokio::sync::RwLock;

use crate::error::LogicError;
use crate::transport::connection::ConnectionInfo;
use crate::transport::settings::SettingConfig;
use crate::utils::file_util;

lazy_static! {
    static ref SETTING_CONFIG: RwLock<Option<SettingConfig>> = RwLock::new(None);
}

pub fn get_setting_from_file() -> Result<SettingConfig, LogicError> {
    let path = file_util::get_setting_file_path();
    let settings = if path.exists() {
        let mut file = File::open(path.display().to_string())?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        serde_json::from_str::<SettingConfig>(content.leak())?
    } else {
        SettingConfig::default()
    };

    Ok(settings)
}

#[tauri::command]
pub async fn get_settings() -> Result<SettingConfig, LogicError> {
    let lock = SETTING_CONFIG.read().await;
    let settings = if lock.is_none() {
        drop(lock);

        let settings = get_setting_from_file()?;
        let mut write_lock = SETTING_CONFIG.write().await;

        let cloned = settings.clone();
        *write_lock = Some(settings);

        cloned
    } else {
        lock.as_ref().unwrap().clone()
    };

    Ok(settings)
}

#[tauri::command]
pub async fn save_settings(setting_config: SettingConfig) -> Result<(), LogicError> {
    let path = file_util::get_setting_file_path();
    let s = serde_json::to_string(&setting_config)?;
    if !path.exists() {
        File::create(path.clone())?;
    }

    fs::write(path, s)?;

    {
        let mut write_lock = SETTING_CONFIG.write().await;
        *write_lock = Some(setting_config);
    }

    debug!("Save settings");

    Ok(())
}

#[tauri::command]
pub fn save_connection(connection: ConnectionInfo) -> Result<(), LogicError> {
    let mut dir = file_util::get_config_dir_path();

    let digest = md5::compute(&connection.name);
    dir.push(format!("{:x}", digest));
    let mut file = if dir.exists() {
        OpenOptions::new().write(true).open(dir)?
    } else {
        File::create(dir)?
    };

    let json = serde_json::to_string(&connection)?;
    let info = BASE64_STANDARD.encode(json.as_bytes());
    file.write_all(info.as_bytes())?;

    Ok(())
}

#[tauri::command]
pub fn remove_connection(name: String) -> Result<(), LogicError> {
    let mut dir = file_util::get_config_dir_path();

    let digest = md5::compute(&name);
    dir.push(format!("{:x}", digest));

    if dir.exists() {
        fs::remove_file(dir)?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_connection_list() -> Result<Vec<ConnectionInfo>, LogicError> {
    let dir = file_util::get_config_dir_path();

    let mut result = Vec::new();
    if dir.exists() {
        let entries = fs::read_dir(dir)?;
        for entry in entries {
            let path = entry?.path();
            if !path.is_dir() {
                let mut file = File::open(path)?;
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                let json = BASE64_STANDARD.decode(content)?;
                let info = serde_json::from_slice::<ConnectionInfo>(json.as_slice())?;
                result.push(info);
            }
        }
    }

    Ok(result)
}

#[tauri::command]
pub fn export_connection(filepath: String) -> Result<(), LogicError> {
    let list = get_connection_list()?;

    let s = serde_json::to_string(&list)?;
    let content = BASE64_STANDARD.encode(s.as_bytes());

    let path = Path::new(&filepath);
    let mut file = if !path.exists() {
        File::create(path)?
    } else {
        File::options()
            .write(true)
            .open(path)?
    };

    file.write_all(content.as_bytes())?;
    Ok(())
}

#[tauri::command]
pub fn import_connection(filepath: String) -> Result<(), LogicError> {
    let path = Path::new(&filepath);
    if !path.exists() {
        return Err(LogicError::ResourceNotExist("File not exists"))
    }

    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let s = BASE64_STANDARD.decode(content)?;
    let list = serde_json::from_slice::<Vec<ConnectionInfo>>(s.as_slice())?;

    for info in list {
        save_connection(info)?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_app_version(app: AppHandle) -> String {
    let conf = app.package_info();
    let version = conf.version.clone();
    version.to_string()
}