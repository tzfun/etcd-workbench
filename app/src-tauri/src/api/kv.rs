use std::str::FromStr;
use log::warn;
use crate::error::LogicError;
use crate::etcd;
use crate::transport::kv::{KVPutResult, SearchResult, SerializableKeyValue};

#[tauri::command]
pub async fn kv_get_all_keys(session: i32) -> Result<Vec<SerializableKeyValue>, LogicError> {
    let mut connector = etcd::get_connector(&session)?;
    let keys = connector.kv_get_all_keys().await?;
    Ok(keys)
}

#[tauri::command]
pub async fn kv_get_all_keys_paging(session: i32, cursor_key: String, limit: i64) -> Result<Vec<SerializableKeyValue>, LogicError> {
    let mut connector = etcd::get_connector(&session)?;
    let keys = connector.kv_get_all_keys_paging(cursor_key, limit).await?;
    Ok(keys)
}

#[tauri::command]
pub async fn kv_get(session: i32, key: String) -> Result<SerializableKeyValue, LogicError> {
    let mut connector = etcd::get_connector(&session)?;
    let mut kv = connector.kv_get(key).await?;
    if kv.lease.ne("0") {
        let lease_id = i64::from_str(kv.lease.as_str()).unwrap();
        let info = connector.lease_get_simple_info(lease_id).await?;
        kv.lease_info = Some(info)
    }
    Ok(kv)
}

#[tauri::command]
pub async fn kv_get_by_version(session: i32, key: String, version: i64) -> Result<SerializableKeyValue, LogicError> {
    let mut connector = etcd::get_connector(&session)?;
    let kv = connector.kv_get_by_version(key, version).await?;
    Ok(kv)
}

#[tauri::command]
pub async fn kv_get_history_versions(session: i32, key: String, start: i64, end: i64) -> Result<Vec<i64>, LogicError> {
    let mut connector = etcd::get_connector(&session)?;
    let versions = connector.kv_get_history_versions(key, start, end).await?;
    Ok(versions)
}

#[tauri::command]
pub async fn kv_get_with_prefix(session: i32, prefix: String) -> Result<SearchResult, LogicError> {
    let mut connector = etcd::get_connector(&session)?;
    let result = connector.kv_get_with_prefix(prefix).await?;
    Ok(result)
}

#[tauri::command]
pub async fn kv_put(session: i32, key: String, value: Vec<u8>, version: i64, ttl: Option<i64>) -> Result<KVPutResult, LogicError> {
    let mut connector = etcd::get_connector(&session)?;

    let response = connector.kv_get_request(key.clone(), None).await?;
    if !response.kvs().is_empty() {
        let kv = &response.kvs()[0];
        if version != kv.version() {
            return Ok(KVPutResult {
                success: false,
                exist_value: Some(Vec::from(kv.value())),
                exist_version: Some(kv.version())
            })
        }
    }

    connector.kv_put(
        key,
        value,
        ttl,
    ).await?;

    Ok(KVPutResult {
        success: true,
        exist_value: None,
        exist_version: None,
    })
}

#[tauri::command]
pub async fn kv_put_with_lease(session: i32, key: String, value: Vec<u8>, lease: String) -> Result<(), LogicError> {
    let mut connector = etcd::get_connector(&session)?;
    let lease = i64::from_str(&lease).map_err(|e| {
        warn!("ttl parse error: {e}");
        LogicError::ArgumentError
    })?;
    connector.kv_put_with_lease(key, value, lease).await?;
    Ok(())
}

#[tauri::command]
pub async fn kv_delete(session: i32, keys: Vec<String>) -> Result<usize, LogicError> {
    let mut connector = etcd::get_connector(&session)?;
    let size = connector.kv_delete(keys).await?;
    Ok(size)
}