use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::{fs, io};

use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use lazy_static::lazy_static;
use log::{debug, warn};
use tauri::AppHandle;
use tokio::sync::RwLock;

use crate::error::LogicError;
use crate::transport::connection::ConnectionInfo;
use crate::transport::settings::SettingConfig;
use crate::utils::{aes_util, file_util};

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
    let new_key = &setting_config.connection_conf_encrypt_key;
    if new_key.as_bytes().len() != aes_util::LENGTH_16 {
        return Err(LogicError::ArgumentError);
    }

    let old_key = get_settings().await?.connection_conf_encrypt_key;
    if old_key.ne(new_key) {
        restore_connections(old_key.as_bytes(), new_key.as_bytes())?;
    }

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

fn restore_connections(old_key: &[u8], new_key: &[u8]) -> io::Result<()> {
    let dir = file_util::get_config_dir_path();
    if dir.exists() {
        let entries = fs::read_dir(dir)?;
        for entry in entries {
            let path = entry?.path();
            if !path.is_dir() {
                let mut file = OpenOptions::new().read(true).write(true).open(path)?;
                let mut content = vec![];
                file.read_to_end(&mut content)?;

                if let Ok(data) = aes_util::reencrypt_128(content, old_key, new_key) {
                    file.write_all(data.as_slice())?;
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "parse config error",
                    ));
                }
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn save_connection(connection: ConnectionInfo) -> Result<(), LogicError> {
    let mut dir = file_util::get_config_dir_path();

    let digest = md5::compute(&connection.name);
    dir.push(format!("{:x}", digest));
    let mut file = if dir.exists() {
        OpenOptions::new().write(true).open(dir)?
    } else {
        File::create(dir)?
    };

    let json = serde_json::to_string(&connection)?;
    let key = get_settings().await?.connection_conf_encrypt_key;

    let data = aes_util::encrypt_128(key.as_bytes(), json)?;

    file.write_all(data.as_slice())?;

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
pub async fn get_connection_list() -> Result<Vec<ConnectionInfo>, LogicError> {
    let dir = file_util::get_config_dir_path();

    let mut result = Vec::new();
    if dir.exists() {
        let settings = get_settings().await?;
        let key = settings.connection_conf_encrypt_key.as_bytes();
        let entries = fs::read_dir(dir)?;
        for entry in entries {
            let path = entry?.path();
            if !path.is_dir() {
                let mut file = File::open(path.clone())?;
                let mut content = vec![];
                file.read_to_end(&mut content)?;

                if let Ok(data) = aes_util::decrypt_128(key, content) {
                    if let Ok(info) = serde_json::from_slice::<ConnectionInfo>(data.as_slice()) {
                        result.push(info);
                    } else {
                        let filepath: String = path.to_string_lossy().to_string();
                        warn!("read connection conf failed with json decode, file will be removed. {}", filepath);
                        let _ = fs::remove_file(path);
                    }
                } else {
                    let filepath: String = path.to_string_lossy().to_string();
                    warn!(
                        "read connection conf failed with aes decrypt, file will be removed. {}",
                        filepath
                    );
                    let _ = fs::remove_file(path);
                }
            }
        }
    }

    Ok(result)
}

#[tauri::command]
pub async fn export_connection(filepath: String) -> Result<(), LogicError> {
    let list = get_connection_list().await?;

    let s = serde_json::to_string(&list)?;
    let content = BASE64_STANDARD.encode(s.as_bytes());

    let path = Path::new(&filepath);
    let mut file = if !path.exists() {
        File::create(path)?
    } else {
        File::options().write(true).open(path)?
    };

    file.write_all(content.as_bytes())?;
    Ok(())
}

#[tauri::command]
pub async fn import_connection(filepath: String) -> Result<(), LogicError> {
    let path = Path::new(&filepath);
    if !path.exists() {
        return Err(LogicError::ResourceNotExist("File not exists"));
    }

    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let s = BASE64_STANDARD.decode(content).map_err(|e| {
        debug!("Failed to decode file. {}", e);
        LogicError::MsgError("Failed to decode file.".to_string())
    })?;
    let list = serde_json::from_slice::<Vec<ConnectionInfo>>(s.as_slice())?;

    for info in list {
        save_connection(info).await?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_app_version(app: AppHandle) -> String {
    let conf = app.package_info();
    let version = conf.version.clone();
    version.to_string()
}
