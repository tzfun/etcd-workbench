use std::fs::File;
use std::io::Read;
use std::fs;

use lazy_static::lazy_static;
use log::debug;
use tauri::updater::{Error, UpdateResponse};
use tauri::{AppHandle, Wry};
use tokio::sync::{Mutex, RwLock};

use crate::api::connection::restore_connections;
use crate::error::LogicError;
use crate::transport::settings::{GlobalStoreConfig, SettingConfig, UpdateManifest, UpdateSource};
use crate::utils::{aes_util, file_util};

/// 从Github拉取更新信息，并从Github下载更新
static UPDATE_SOURCE_GITHUB: &str = "https://tzfun.github.io/etcd-workbench/etcd-workbench-update.json";
/// 从Github拉取更新信息，并从Gitee下载更新
static UPDATE_SOURCE_GITEE_CHECK_FROM_GITHUB: &str = "https://tzfun.gitee.io/etcd-workbench/etcd-workbench-update-gitee.json";
/// 从Gitee拉取更新信息，并从Gitee下载更新
static UPDATE_SOURCE_GITEE_CHECK_FROM_GITEE: &str = "https://gitee.com/tzfun/etcd-workbench/raw/master/docs/etcd-workbench-update-gitee.json";

lazy_static! {
    static ref SETTING_CONFIG: RwLock<Option<SettingConfig>> = RwLock::new(None);
    static ref GLOBAL_STORE_CONFIG: RwLock<Option<GlobalStoreConfig>> = RwLock::new(None);
    static ref UPDATE_RESULT: Mutex<Option<UpdateResponse<Wry>>> = Mutex::new(None);
}

/// 从文件中读取设置数据
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

/// 从文件中读取全局存储数据
pub fn get_global_store_from_file() -> Result<GlobalStoreConfig, LogicError> {
    let path = file_util::get_global_store_file_path();
    let store = if path.exists() {
        let mut file = File::open(path.display().to_string())?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        serde_json::from_str::<GlobalStoreConfig>(content.leak())?
    } else {
        GlobalStoreConfig::default()
    };

    Ok(store)
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
pub async fn get_global_store() -> Result<GlobalStoreConfig, LogicError> {
    let lock = GLOBAL_STORE_CONFIG.read().await;
    let store = if lock.is_none() {
        drop(lock);

        let store = get_global_store_from_file()?;
        let mut write_lock = GLOBAL_STORE_CONFIG.write().await;

        let cloned = store.clone();
        *write_lock = Some(store);

        cloned
    } else {
        lock.as_ref().unwrap().clone()
    };

    Ok(store)
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

#[tauri::command]
pub async fn save_global_store(store: GlobalStoreConfig) -> Result<(), LogicError> {
    let path = file_util::get_global_store_file_path();
    let s = serde_json::to_string(&store)?;
    if !path.exists() {
        File::create(path.clone())?;
    }
    fs::write(path, s)?;

    {
        let mut write_lock = GLOBAL_STORE_CONFIG.write().await;
        *write_lock = Some(store);
    }

    debug!("Save global store");

    Ok(())
}

#[tauri::command]
pub fn get_app_version(app: AppHandle) -> String {
    let conf = app.package_info();
    let version = conf.version.clone();
    version.to_string()
}

#[tauri::command]
#[cfg(debug_assertions)]
pub fn is_debug_model() -> bool {
    true
}

#[tauri::command]
#[cfg(not(debug_assertions))]
pub fn is_debug_model() -> bool {
    false
}

#[tauri::command]
pub async fn check_update(app_handle: AppHandle,) -> Result<Option<UpdateManifest>, LogicError> {
    let mut update_builder = tauri::updater::builder(app_handle);

    let setting = get_settings().await?;
    update_builder = match setting.update_source {
        UpdateSource::Github => {
            update_builder.endpoints(&[String::from(UPDATE_SOURCE_GITHUB)])
        },
        UpdateSource::Gitee => {
            //  从Gitee读取，从Gitee下载
            update_builder = update_builder.endpoints(&[String::from(UPDATE_SOURCE_GITEE_CHECK_FROM_GITEE)]);
            //  从GitHub读取，从Gitee下载
            update_builder = update_builder.endpoints(&[String::from(UPDATE_SOURCE_GITEE_CHECK_FROM_GITHUB)]);
            //  为了避免国内镜像连接失效，保底从GitHub读取，从GitHub下载
            update_builder.endpoints(&[String::from(UPDATE_SOURCE_GITHUB)])
        }
    };

    let update = update_builder.check().await?;
    if update.is_update_available() {
        let version = String::from(update.latest_version());
        let body = update.body().map(|s| s.clone()).unwrap();
        let date = update.date().unwrap().unix_timestamp();

        let mut lock = UPDATE_RESULT.lock().await;
        *lock = Some(update);

        return Ok(Some(UpdateManifest {
            version,
            date,
            body
        }))
    }
    
    Ok(None)
}


#[tauri::command]
pub async fn install_update() -> Result<(), LogicError> {
    let mut lock = UPDATE_RESULT.lock().await;
    let update = lock.take();
    drop(lock);
    if update.is_none() {
        return Err(LogicError::UpdateError(Error::UpToDate))
    }
    let update = update.unwrap();
    update.download_and_install().await?;
    Ok(())
}