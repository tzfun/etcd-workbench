use crate::{
    error::LogicError,
    transport::{
        event::UpdateDownloadingProgressEvent,
        settings::{UpdateManifest, UpdateSource},
    },
};
use lazy_static::lazy_static;
use log::info;
use tauri::{updater::UpdateResponse, AppHandle, Manager, UpdaterEvent, Window, Wry};
use tokio::sync::{Mutex, RwLock};

use super::settings::get_settings;

/// 从Github拉取更新信息，并从Github下载更新
static UPDATE_SOURCE_GITHUB: &str =
    "https://tzfun.github.io/etcd-workbench/etcd-workbench-update.json";
/// 从Github拉取更新信息，并从Gitee下载更新
static UPDATE_SOURCE_GITEE_CHECK_FROM_GITHUB: &str =
    "https://tzfun.gitee.io/etcd-workbench/etcd-workbench-update-gitee.json";
/// 从Gitee拉取更新信息，并从Gitee下载更新
static UPDATE_SOURCE_GITEE_CHECK_FROM_GITEE: &str =
    "https://gitee.com/tzfun/etcd-workbench/raw/master/docs/etcd-workbench-update-gitee.json";

lazy_static! {
    static ref UPDATE_RESULT: Mutex<Option<UpdateResponse<Wry>>> = Mutex::new(None);
    static ref UPDATE_CHECK_SOURCE: RwLock<String> = RwLock::new(String::from("main"));
}

#[tauri::command]
pub async fn check_update(
    app_handle: AppHandle,
    window: Window,
) -> Result<bool, LogicError> {
    check_update_with_source(app_handle, String::from(window.label())).await
}

pub async fn check_update_with_source(
    app_handle: AppHandle,
    source: String,
) -> Result<bool, LogicError> {
    let mut update_builder = tauri::updater::builder(app_handle.clone());

    let setting = get_settings().await?;
    update_builder = match setting.update_source {
        UpdateSource::Github => update_builder.endpoints(&[String::from(UPDATE_SOURCE_GITHUB)]),
        UpdateSource::Gitee => {
            //  从Gitee读取，从Gitee下载
            update_builder =
                update_builder.endpoints(&[String::from(UPDATE_SOURCE_GITEE_CHECK_FROM_GITEE)]);
            //  从GitHub读取，从Gitee下载
            update_builder =
                update_builder.endpoints(&[String::from(UPDATE_SOURCE_GITEE_CHECK_FROM_GITHUB)]);
            //  为了避免国内镜像连接失效，保底从GitHub读取，从GitHub下载
            update_builder.endpoints(&[String::from(UPDATE_SOURCE_GITHUB)])
        }
    };

    let update = update_builder.check().await?;
    let available = update.is_update_available();
    if available {
        if get_settings().await?.auto_update {
            tokio::spawn(async move {
                show_main_window(&app_handle).await;
                let _ = update.download_and_install().await;
            });
        } else {
            let mut source_lock = UPDATE_CHECK_SOURCE.write().await;
            *source_lock = source;
            drop(source_lock);
            
            let mut lock = UPDATE_RESULT.lock().await;
            *lock = Some(update);
            drop(lock);
        }
    }

    Ok(available)
}

#[tauri::command]
pub async fn install_update(app_handle: AppHandle,) -> Result<(), LogicError> {
    let mut lock = UPDATE_RESULT.lock().await;
    let update = lock.take();
    drop(lock);
    if update.is_none() {
        return Err(LogicError::UpdateError(tauri::updater::Error::UpToDate));
    }
    let update = update.unwrap();
    show_main_window(&app_handle).await;
    update.download_and_install().await?;
    Ok(())
}

async fn show_main_window(app_handle: &AppHandle) {
    if let Some(window) = app_handle.get_window("setting") {
        let _ = window.hide();
    }
    let _ = app_handle.get_window("main").unwrap().set_focus();
}

pub fn handle_updater_event(app: &AppHandle, updater_event: UpdaterEvent) {
    match updater_event {
        tauri::UpdaterEvent::UpdateAvailable {
            body,
            date,
            version,
        } => {
            info!("update available body='{}', date={:?}, version={}", body, date, version);
            let date = date.map(|date_time| date_time.unix_timestamp());
            let app = app.clone();
            tokio::spawn(async move {
                let source_lock = UPDATE_CHECK_SOURCE.read().await;
                let source = source_lock.clone();
                drop(source_lock);
    
                let _ = app.emit_all(
                    "updateAvailable",
                    UpdateManifest {
                        version,
                        date,
                        body,
                        source,
                    });
            });
        }
        // Emitted when the download is about to be started.
        tauri::UpdaterEvent::Pending => {
            info!("update is pending!");
            let _ = app.emit_all("updatePending", ());
        }
        tauri::UpdaterEvent::DownloadProgress {
            chunk_length,
            content_length,
        } => {
            let _ = app.emit_all(
                "updateDownloadingProgress",
                UpdateDownloadingProgressEvent {
                    chunk_length,
                    content_length,
                },
            );
        }
        // Emitted when the download has finished and the update is about to be installed.
        tauri::UpdaterEvent::Downloaded => {
            info!("update has been downloaded!");
            let _ = app.emit_all("updateDownloaded", ());
        }
        // Emitted when the update was installed. You can then ask to restart the app.
        tauri::UpdaterEvent::Updated => {
            info!("app has been updated");
            let _ = app.emit_all("updateInstalled", ());
        }
        // Emitted when the app already has the latest version installed and an update is not needed.
        tauri::UpdaterEvent::AlreadyUpToDate => {
            info!("app is already up to date");
        }
        // Emitted when there is an error with the updater. We suggest to listen to this event even if the default dialog is enabled.
        tauri::UpdaterEvent::Error(error) => {
            info!("failed to update: {}", error);
            let _ = app.emit_all("updateErrors", error);
        }
    }
}
