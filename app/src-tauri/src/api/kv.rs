use crate::api::settings::get_settings;
use crate::error::LogicError;
use crate::etcd;
use crate::transport::event::{KVBatchImportAndExportEvent, KVRenameDirEvent};
use crate::transport::kv::{
    KVPutResult, PutStrategy, RenameAction, SearchResult, SerializableKeyValue,
};
use crate::utils::{hex_to_vec, vec_to_hex};
use etcd_client::{GetOptions, PutOptions};
use log::warn;
use std::io;
use std::path::Path;
use std::str::FromStr;
use tauri::{AppHandle, Manager};
use tokio::fs::{self, File};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

const RENAME_DIR_EVENT: &str = "renameDirEvent";
const RENAME_DIR_START_EVENT: &str = "renameDirStartEvent";
const RENAME_DIR_END_EVENT: &str = "renameDirEndEvent";
const RENAME_DIR_ERR_EVENT: &str = "renameDirErrEvent";

const BATCH_EXPORT_EVENT: &str = "batchExportEvent";
const BATCH_EXPORT_START_EVENT: &str = "batchExportStartEvent";
const BATCH_EXPORT_END_EVENT: &str = "batchExportEndEvent";
const BATCH_EXPORT_ERR_EVENT: &str = "batchExportErrEvent";

const BATCH_IMPORT_EVENT: &str = "batchImportEvent";
const BATCH_IMPORT_START_EVENT: &str = "batchImportStartEvent";
const BATCH_IMPORT_END_EVENT: &str = "batchImportEndEvent";
const BATCH_IMPORT_ERR_EVENT: &str = "batchImportErrEvent";

#[tauri::command]
pub async fn kv_get_all_keys(session: i32) -> Result<Vec<SerializableKeyValue>, LogicError> {
    let mut connector = etcd::get_connector(&session)?;
    let keys = connector.kv_get_all_keys().await?;
    Ok(keys)
}

#[tauri::command]
pub async fn kv_get_all_keys_paging(
    session: i32,
    cursor_key: String,
    limit: i64,
) -> Result<Vec<SerializableKeyValue>, LogicError> {
    let mut connector = etcd::get_connector(&session)?;
    let keys = connector.kv_get_all_keys_paging(cursor_key, limit).await?;
    Ok(keys)
}

#[tauri::command]
pub async fn kv_get(
    session: i32,
    key: String,
    key_bytes: Option<Vec<u8>>,
) -> Result<SerializableKeyValue, LogicError> {
    let mut connector = etcd::get_connector(&session)?;
    let k = if let Some(key_bytes) = key_bytes {
        key_bytes
    } else {
        key.into()
    };
    let mut kv = connector.kv_get(k).await?;
    if kv.lease.ne("0") {
        let lease_id = i64::from_str(kv.lease.as_str()).unwrap();
        let info = connector.lease_get_simple_info(lease_id).await?;
        kv.lease_info = Some(info)
    }
    Ok(kv)
}

#[tauri::command]
pub async fn kv_get_by_version(
    session: i32,
    key: String,
    key_bytes: Option<Vec<u8>>,
    version: i64,
) -> Result<SerializableKeyValue, LogicError> {
    let mut connector = etcd::get_connector(&session)?;
    let k = if let Some(key_bytes) = key_bytes {
        key_bytes
    } else {
        key.into()
    };

    let kv = connector.kv_get_by_version(k, version).await?;
    Ok(kv)
}

#[tauri::command]
pub async fn kv_get_history_versions(
    session: i32,
    key: String,
    key_bytes: Option<Vec<u8>>,
    start: i64,
    end: i64,
) -> Result<Vec<i64>, LogicError> {
    let mut connector = etcd::get_connector(&session)?;
    let k = if let Some(key_bytes) = key_bytes {
        key_bytes
    } else {
        key.into()
    };
    let versions = connector.kv_get_history_versions(k, start, end).await?;
    Ok(versions)
}

#[tauri::command]
pub async fn kv_get_with_prefix(session: i32, prefix: String) -> Result<SearchResult, LogicError> {
    let mut connector = etcd::get_connector(&session)?;
    let result = connector.kv_get_with_prefix(prefix).await?;
    Ok(result)
}

#[tauri::command]
pub async fn kv_put(
    session: i32,
    key: String,
    value: Vec<u8>,
    version: i64,
    ttl: Option<i64>,
) -> Result<KVPutResult, LogicError> {
    let mut connector = etcd::get_connector(&session)?;

    //  version用于冲突判断，如果当前版本号和最新的版本号不匹配则需要客户端解决冲突
    //  如果version < 0则直接插入无需判断
    if version >= 0 {
        let response = connector.kv_get_request(key.clone(), None).await?;
        if !response.kvs().is_empty() {
            let kv = &response.kvs()[0];
            if version != kv.version() {
                return Ok(KVPutResult {
                    success: false,
                    final_kv: None,
                    exist_value: Some(Vec::from(kv.value())),
                    exist_version: Some(kv.version()),
                });
            }
        }
    }

    let final_kv = connector.kv_put(key, value, ttl).await?;

    Ok(KVPutResult {
        success: true,
        final_kv,
        exist_value: None,
        exist_version: None,
    })
}

#[tauri::command]
pub async fn kv_put_with_lease(
    session: i32,
    key: String,
    value: Vec<u8>,
    lease: String,
) -> Result<(), LogicError> {
    let mut connector = etcd::get_connector(&session)?;
    let lease = i64::from_str(&lease).map_err(|e| {
        warn!("ttl parse error: {e}");
        LogicError::ArgumentError
    })?;
    connector.kv_put_with_lease(key, value, lease).await?;
    Ok(())
}

#[tauri::command]
pub async fn kv_delete(
    session: i32,
    keys: Vec<String>,
    mut key_bytes: Vec<Vec<u8>>,
) -> Result<usize, LogicError> {
    let mut connector = etcd::get_connector(&session)?;
    for key_str in keys {
        key_bytes.push(key_str.into());
    }
    let size = connector.kv_delete(key_bytes).await?;
    Ok(size)
}

#[tauri::command]
pub async fn kv_search_next_dir(
    session: i32,
    prefix: String,
    include_file: bool,
) -> Result<Vec<String>, LogicError> {
    let mut connector = etcd::get_connector(&session)?;
    connector.kv_search_next_dir(prefix, include_file).await
}

#[tauri::command]
pub async fn kv_rename_dir(
    session: i32,
    origin_prefix: String,
    new_prefix: String,
    delete_origin_keys: bool,
    put_strategy: PutStrategy,
    app_handle: AppHandle,
) -> Result<(), LogicError> {
    let mut connector = etcd::get_connector(&session)?;
    let full_origin_prefix = connector.fill_prefix_namespace(origin_prefix.clone());
    let client = connector.inner();
    let count_resp = client
        .kv_get_request(
            full_origin_prefix,
            Some(GetOptions::new().with_prefix().with_count_only()),
        )
        .await?;
    let count_limit = get_settings().await?.kv_dir_rename_keys_limit;
    let count = count_resp.count();
    if count > count_limit {
        return Err(LogicError::LimitedError(count));
    }

    drop(connector);

    tauri::async_runtime::spawn(async move {
        if let Err(e) = rename_dir(
            session,
            origin_prefix,
            new_prefix,
            delete_origin_keys,
            put_strategy,
            &app_handle,
            count,
        )
        .await
        {
            log::error!("rename dir error: {:?}", e);
            let _ = app_handle.emit_to("main", RENAME_DIR_ERR_EVENT, ());
        }
    });
    Ok(())
}

async fn rename_dir(
    session: i32,
    origin_prefix: String,
    new_prefix: String,
    delete_origin_keys: bool,
    put_strategy: PutStrategy,
    app_handle: &AppHandle,
    count: i64,
) -> Result<(), LogicError> {
    let _ = app_handle.emit_to("main", RENAME_DIR_START_EVENT, count);

    let mut connector = etcd::get_connector(&session)?;
    let full_origin_prefix = connector.fill_prefix_namespace(origin_prefix.clone());
    let full_origin_prefix_len = full_origin_prefix.len();
    let full_new_prefix = connector.fill_prefix_namespace(new_prefix.clone());
    let namespace_len = connector.namespace_bytes_len();
    let client = connector.inner();
    let mut get_resp = client
        .kv_get_request(full_origin_prefix, Some(GetOptions::new().with_prefix()))
        .await?;

    for kv in get_resp.take_kvs() {
        let key_suffix = &kv.key()[full_origin_prefix_len..];

        let mut full_new_key = full_new_prefix.clone();
        full_new_key.extend_from_slice(key_suffix);

        //  如果使用重命名策略，则需要检查新key是否已存在
        if put_strategy == PutStrategy::Rename {
            if let Ok(exist_resp) = client
                .kv_get_request(
                    full_new_key.clone(),
                    Some(GetOptions::new().with_keys_only()),
                )
                .await
            {
                if exist_resp.count() == 1 {
                    full_new_key = PutStrategy::rename(exist_resp.kvs()[0].key());
                }
            }
        }

        let mut new_key = full_new_key.clone();
        new_key.drain(0..namespace_len); // 删除namespace前缀

        let put_result = client
            .kv_put_request(
                full_new_key,
                kv.value().to_vec(),
                Some(PutOptions::new().with_lease(kv.lease())),
            )
            .await;
        if let Err(e) = put_result {
            let event = KVRenameDirEvent {
                key: new_key,
                success: false,
                action: RenameAction::Put,
                failed_msg: Some(e.to_string()),
            };
            let _ = app_handle.emit_to("main", RENAME_DIR_EVENT, event);
            break;
        } else {
            let event = KVRenameDirEvent {
                key: new_key,
                success: true,
                action: RenameAction::Put,
                failed_msg: None,
            };
            let _ = app_handle.emit_to("main", RENAME_DIR_EVENT, event);
        };

        if delete_origin_keys {
            let full_origin_key = kv.key().to_vec();
            let mut origin_key = full_origin_key.clone();
            origin_key.drain(0..namespace_len); // 删除namespace前缀

            // 删除原有的key
            if let Err(e) = client.kv_delete_request(full_origin_key, None).await {
                let event = KVRenameDirEvent {
                    key: origin_key,
                    success: false,
                    action: RenameAction::Delete,
                    failed_msg: Some(e.to_string()),
                };
                let _ = app_handle.emit_to("main", RENAME_DIR_EVENT, event);
                break;
            } else {
                let event = KVRenameDirEvent {
                    key: origin_key,
                    success: true,
                    action: RenameAction::Delete,
                    failed_msg: None,
                };
                let _ = app_handle.emit_to("main", RENAME_DIR_EVENT, event);
            }
        }
    }

    let _ = app_handle.emit_to("main", RENAME_DIR_END_EVENT, ());

    Ok(())
}

#[tauri::command]
pub async fn kv_batch_export(
    app_handle: AppHandle,
    session: i32,
    keys: Vec<Vec<u8>>,
    target_path: String,
) -> Result<(), LogicError> {
    let path = Path::new(&target_path);
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).await?;
        }
    }
    if keys.is_empty() {
        return Err(LogicError::ArgumentError);
    }

    let c = etcd::get_connector(&session)?;
    drop(c);

    tauri::async_runtime::spawn(async move {
        if let Err(e) = batch_export(&app_handle, session, keys, target_path).await {
            log::error!("batch export error: {:?}", e);
            let _ = app_handle.emit_to("main", BATCH_EXPORT_ERR_EVENT, format!("{:?}", e));
        }
    });

    Ok(())
}

async fn batch_export(
    app_handle: &AppHandle,
    session: i32,
    keys: Vec<Vec<u8>>,
    target_path: String,
) -> Result<(), LogicError> {
    let _ = app_handle.emit_to("main", BATCH_EXPORT_START_EVENT, ());

    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(target_path)
        .await?;

    let mut connector = etcd::get_connector(&session)?;
    for k in keys {
        let key = connector.fill_prefix_namespace(k.clone());
        let response = connector.inner().kv_get_request(key, None).await?;
        let event = if response.count() == 1 {
            let value = &response.kvs()[0];
            file.write(vec_to_hex(&k).as_bytes()).await?;
            file.write(b"\n").await?;

            file.write(vec_to_hex(value.value()).as_bytes()).await?;
            file.write(b"\n").await?;

            KVBatchImportAndExportEvent {
                success: true,
                key: Some(k),
                failed_msg: None,
            }
        } else {
            KVBatchImportAndExportEvent {
                success: false,
                key: Some(k),
                failed_msg: Some(format!("The number of entries read from the remote key is incorrect: expected 1, received {}.", response.count())),
            }
        };
        let _ = app_handle.emit_to("main", BATCH_EXPORT_EVENT, event);
    }
    let _ = app_handle.emit_to("main", BATCH_EXPORT_END_EVENT, ());

    Ok(())
}

#[tauri::command]
pub async fn kv_batch_import(
    app_handle: AppHandle,
    session: i32,
    target_path: String,
    prefix: Option<String>,
    put_strategy: PutStrategy,
) -> Result<(), LogicError> {
    if put_strategy == PutStrategy::AskMerge {
        return Err(LogicError::ArgumentError);
    }

    let path = Path::new(&target_path);
    if !path.exists() {
        return Err(LogicError::IoError(io::Error::new(
            io::ErrorKind::NotFound,
            "The file does not exist.",
        )));
    }

    let c = etcd::get_connector(&session)?;
    drop(c);

    tauri::async_runtime::spawn(async move {
        if let Err(e) = batch_import(&app_handle, session, target_path, prefix, put_strategy).await
        {
            log::error!("batch import error: {:?}", e);
            let _ = app_handle.emit_to("main", BATCH_IMPORT_ERR_EVENT, format!("{:?}", e));
        }
    });

    Ok(())
}

async fn batch_import(
    app_handle: &AppHandle,
    session: i32,
    target_path: String,
    prefix: Option<String>,
    put_strategy: PutStrategy,
) -> Result<(), LogicError> {
    let _ = app_handle.emit_to("main", BATCH_IMPORT_START_EVENT, ());
    let file = File::open(target_path).await?;
    let reader = BufReader::new(file);

    let mut connector = etcd::get_connector(&session)?;

    let mut lines = reader.lines();
    loop {
        let key = lines.next_line().await?;
        if key.is_none() {
            break;
        }
        let value = lines.next_line().await?;
        if value.is_none() {
            break;
        }

        let key = hex_to_vec(key.unwrap());
        let value = hex_to_vec(value.unwrap());

        if let Err(e) = key {
            let event = KVBatchImportAndExportEvent {
                success: false,
                key: None,
                failed_msg: Some(e),
            };
            let _ = app_handle.emit_to("main", BATCH_IMPORT_EVENT, event);
            break;
        }
        let mut key = key.unwrap();

        if let Err(e) = value {
            let event = KVBatchImportAndExportEvent {
                success: false,
                key: Some(key),
                failed_msg: Some(e),
            };
            let _ = app_handle.emit_to("main", BATCH_IMPORT_EVENT, event);
            break;
        }
        let value = value.unwrap();
        if let Some(prefix) = &prefix {
            key.splice(0..0, prefix.as_bytes().to_vec());
        }

        let mut full_key = connector.fill_prefix_namespace(key.clone());

        match put_strategy {
            PutStrategy::Cover => {}
            PutStrategy::Rename => {
                match connector
                    .inner()
                    .kv_get_request(full_key.clone(), Some(GetOptions::new().with_keys_only()))
                    .await
                {
                    Ok(res) => {
                        if res.count() > 0 {
                            full_key = PutStrategy::rename(&full_key);
                        }
                    }
                    Err(e) => {
                        let event = KVBatchImportAndExportEvent {
                            success: false,
                            key: Some(key),
                            failed_msg: Some(format!("Failed to rename key: {}", e)),
                        };

                        let _ = app_handle.emit_to("main", BATCH_IMPORT_EVENT, event);
                        continue;
                    }
                }
            }
            _ => {
                continue;
            }
        }

        let event = if let Err(e) = connector
            .inner()
            .kv_put_request(full_key, value, None)
            .await
        {
            KVBatchImportAndExportEvent {
                success: false,
                key: Some(key),
                failed_msg: Some(e.to_string()),
            }
        } else {
            KVBatchImportAndExportEvent {
                success: true,
                key: Some(key),
                failed_msg: None,
            }
        };
        let _ = app_handle.emit_to("main", BATCH_IMPORT_EVENT, event);
    }

    let _ = app_handle.emit_to("main", BATCH_IMPORT_END_EVENT, ());

    Ok(())
}
