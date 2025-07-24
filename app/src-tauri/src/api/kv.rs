use crate::api::settings::get_settings;
use crate::error::LogicError;
use crate::etcd;
use crate::transport::kv::{
    KVPutResult, KVRenameDirEvent, PutStrategy, RenameAction, SearchResult, SerializableKeyValue
};
use etcd_client::{GetOptions, PutOptions};
use log::warn;
use std::str::FromStr;
use tauri::{AppHandle, Manager};

const RENAME_DIR_EVENT: &str = "renameDirEvent";
const RENAME_DIR_START_EVENT: &str = "renameDirStartEvent";
const RENAME_DIR_END_EVENT: &str = "renameDirEndEvent";
const RENAME_DIR_ERR_EVENT: &str = "renameDirErrEvent";

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
            if let Ok(exist_resp) = client.kv_get_request(full_new_key.clone(), Some(GetOptions::new().with_keys_only())).await {
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
