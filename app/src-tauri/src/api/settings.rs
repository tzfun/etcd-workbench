use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use base64::Engine;
use base64::prelude::BASE64_STANDARD;

use crate::error::LogicError;
use crate::transport::connection::ConnectionInfo;
use crate::transport::settings::Settings;
use crate::utils::file_util;

#[tauri::command]
pub fn get_settings() -> Result<Settings, LogicError> {
    let mut path = PathBuf::from(file_util::get_storage_dir()?);
    path.push(file_util::SETTINGS_FILE);

    let settings = if path.exists() {
        let mut file = File::open(path.display().to_string())?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        serde_json::from_str::<Settings>(content.leak())?
    } else {
        Settings::default()
    };

    Ok(settings)
}

#[tauri::command]
pub fn save_connection(connection: ConnectionInfo) -> Result<(), LogicError> {
    let mut dir = PathBuf::from(file_util::get_storage_dir()?);
    dir.push(file_util::CONFIG_DIR);
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }

    let digest = md5::compute(&connection.name);
    dir.push(format!("{:x}", digest));
    let mut file = if dir.exists() {
        File::open(dir)?
    } else {
        File::create(dir)?
    };

    let json = serde_json::to_string(&connection)?;
    let info = BASE64_STANDARD.encode(json.as_bytes());
    file.write_all(info.as_bytes())?;

    Ok(())
}

#[tauri::command]
pub fn get_connection_list() -> Result<Vec<ConnectionInfo>, LogicError> {
    let mut dir = PathBuf::from(file_util::get_storage_dir()?);
    dir.push(file_util::CONFIG_DIR);
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