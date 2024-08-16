use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

use base64::Engine;
use base64::prelude::BASE64_STANDARD;

use crate::error::LogicError;
use crate::transport::connection::ConnectionInfo;
use crate::transport::settings::Settings;
use crate::utils::file_util;

#[tauri::command]
pub fn get_settings() -> Result<Settings, LogicError> {
    let path = file_util::get_setting_file_path();

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