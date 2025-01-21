use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::{fs, io, vec};

use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use log::{debug, info, warn};
use tauri::Window;

use crate::error::LogicError;
use crate::etcd;
use crate::etcd::etcd_connector::EtcdConnector;
use crate::etcd::key_monitor::KeyMonitor;
use crate::transport::connection::{Connection, ConnectionInfo, KeyMonitorConfig, SessionData};
use crate::utils::{aes_util, file_util, md5};

use super::settings::get_settings;

#[tauri::command]
pub async fn connect_test(connection: Connection) -> Result<(), LogicError> {
    let connector = EtcdConnector::new(connection).await?;
    connector.test_connection().await?;
    Ok(())
}

#[tauri::command]
pub async fn connect(name: String, connection: Connection, window: Window) -> Result<SessionData, LogicError> {
    let session = etcd::new_connector(name, connection, window).await?;
    info!("New connection: {}", session.id);
    Ok(session)
}

#[tauri::command]
pub async fn disconnect(session: i32) -> Result<(), LogicError> {
    etcd::remove_connector(&session).await;
    info!("Removed connection: {}", session);
    Ok(())
}

pub fn restore_connections(old_key: &[u8], new_key: &[u8]) -> io::Result<()> {
    let dir = file_util::get_conn_config_dir_path();
    if dir.exists() {
        let entries = fs::read_dir(dir)?;
        for entry in entries {
            let path = entry?.path();
            if !path.is_dir() {
                let mut file = File::open(&path)?;
                let mut content = vec![];
                file.read_to_end(&mut content)?;

                fs::remove_file(&path)?;
                let mut file = File::create(&path)?;

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

/// 保存连接信息，继承其他配置项
#[tauri::command]
pub async fn save_connection(name: String, connection: Connection) -> Result<(), LogicError> {
    let mut dir = file_util::get_conn_config_dir_path();
    let key = get_settings().await?.connection_conf_encrypt_key;

    let mut connection_info = ConnectionInfo {
        name,
        connection,
        key_collection: vec![],
        key_monitor_list: vec![],
        key_monitor_paused: false,
    };
    let file_name = md5(&connection_info.name);
    dir.push(file_name);

    //  如果配置已经存在，只更新连接配置项，继承已有的其他配置
    if dir.exists() {
        let mut file = File::open(&dir)?;
        let mut content = vec![];
        file.read_to_end(&mut content)?;
        if let Ok(data) = aes_util::decrypt_128(key.as_bytes(), content) {
            if let Ok(info) = serde_json::from_slice::<ConnectionInfo>(data.as_slice()) {
                connection_info.key_collection = info.key_collection;
                connection_info.key_monitor_list = info.key_monitor_list;
                connection_info.key_monitor_paused = info.key_monitor_paused;
            }
        }

        fs::remove_file(&dir)?;
    }
    let mut file = File::create(dir)?;

    let json = serde_json::to_string(&connection_info)?;

    let data = aes_util::encrypt_128(key.as_bytes(), json)?;

    file.write_all(data.as_slice())?;

    Ok(())
}

//  保存完整的连接info数据
pub async fn save_connection_info(info: ConnectionInfo) -> Result<(), LogicError> {
    let mut dir = file_util::get_conn_config_dir_path();
    let key = get_settings().await?.connection_conf_encrypt_key;

    let file_name = md5(&info.name);
    dir.push(file_name);

    if dir.exists() {
        fs::remove_file(&dir)?;
    }

    let mut file = File::create(dir)?;

    let json = serde_json::to_string(&info)?;

    let data = aes_util::encrypt_128(key.as_bytes(), json)?;

    file.write_all(data.as_slice())?;

    Ok(())
}

#[tauri::command]
pub fn remove_connection(name: String) -> Result<(), LogicError> {
    let mut dir = file_util::get_conn_config_dir_path();

    let file_name = md5(&name);
    dir.push(file_name);

    if dir.exists() {
        fs::remove_file(dir)?;
    }
    Ok(())
}

/// 根据连接名查询已保存的配置
///
/// 如果存在返回一个 Some(ConnectionInfo)
/// 如果不存在返回 None
pub async fn get_connection(name: String) -> Result<Option<ConnectionInfo>, LogicError> {
    let mut dir = file_util::get_conn_config_dir_path();
    let file_name = md5(&name);
    dir.push(file_name);

    if dir.exists() {
        let mut file = OpenOptions::new().read(true).open(dir)?;
        let mut content = vec![];
        file.read_to_end(&mut content)?;

        let key = get_settings().await?.connection_conf_encrypt_key;

        if let Ok(data) = aes_util::decrypt_128(key.as_bytes(), content) {
            if let Ok(info) = serde_json::from_slice::<ConnectionInfo>(data.as_slice()) {
                return Ok(Some(info));
            }
        }
    }
    Ok(None)
}

#[tauri::command]
pub async fn get_connection_list() -> Result<Vec<ConnectionInfo>, LogicError> {
    let dir = file_util::get_conn_config_dir_path();

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
                        debug!("File content decoded: {}", String::from_utf8(data).unwrap());

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
        save_connection_info(info).await?;
    }

    Ok(())
}

#[tauri::command]
pub async fn update_key_collection(
    session: i32,
    key_collection: Vec<String>,
) -> Result<(), LogicError> {
    let result = etcd::get_connection_info_optional(&session);
    if let Some(mut info) = result {
        info.key_collection = key_collection;
        save_connection_info(info.value().clone()).await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn set_key_monitor(
    session: i32,
    key_monitor: KeyMonitorConfig,
) -> Result<(), LogicError> {

    let result = etcd::get_connection_info_optional(&session);
    if let Some(mut info) = result {
        let mut found = false;
        for km in info.key_monitor_list.iter_mut() {
            if km.key.eq(&key_monitor.key) {
                km.merge(&key_monitor);
                found = true;
                break;
            }
        }

        if !found {
            info.key_monitor_list.push(key_monitor.clone());
        }
        save_connection_info(info.value().clone()).await?;
    }

    let lock_ref = etcd::get_key_monitor(&session);
    let lock = lock_ref.value().clone();
    KeyMonitor::set_config(lock, key_monitor).await;
    Ok(())
}

#[tauri::command]
pub async fn remove_key_monitor(
    session: i32,
    key: String,
) -> Result<(), LogicError> {
    let result = etcd::get_connection_info_optional(&session);
    if let Some(mut info) = result {
        info.key_monitor_list.retain(|c| c.key.ne(&key));
        
        save_connection_info(info.value().clone()).await?;
    }

    let lock_ref = etcd::get_key_monitor(&session);
    let lock = lock_ref.value().clone();
    KeyMonitor::remove_config(lock, &key).await;
    Ok(())
}

#[tauri::command]
pub async fn key_monitor_toggle_pause(session: i32, state: bool) -> Result<(), LogicError> {
    let result = etcd::get_connection_info_optional(&session);
    if let Some(mut info) = result {
        info.key_monitor_paused = state;
        save_connection_info(info.value().clone()).await?;
    }
    let lock_ref = etcd::get_key_monitor(&session);
    let lock = lock_ref.value().clone();
    KeyMonitor::toggle_pause(lock, state).await;
    Ok(())
}