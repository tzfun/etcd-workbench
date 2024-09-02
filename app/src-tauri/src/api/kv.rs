use std::str::FromStr;
use log::warn;
use crate::error::LogicError;
use crate::etcd;
use crate::transport::kv::SerializableKeyValue;

#[tauri::command]
pub async fn kv_get_all_keys(session: i32) -> Result<Vec<SerializableKeyValue>, LogicError> {
    let connector = etcd::get_connector(&session)?;
    let keys = connector.kv_get_all_keys().await?;
    Ok(keys)
}

#[tauri::command]
pub async fn kv_get_all_keys_paging(session: i32, mut cursor_key: String, limit: i64) -> Result<Vec<SerializableKeyValue>, LogicError> {
    let connector = etcd::get_connector(&session)?;
    let keys = connector.kv_get_all_keys_paging(cursor_key, limit).await?;
    Ok(keys)
}

#[tauri::command]
pub async fn kv_get(session: i32, key: String) -> Result<SerializableKeyValue, LogicError> {
    let connector = etcd::get_connector(&session)?;
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
    let connector = etcd::get_connector(&session)?;
    let kv = connector.kv_get_by_version(key, version).await?;
    Ok(kv)
}

#[tauri::command]
pub async fn kv_put(session: i32, key: String, value: Vec<u8>, ttl: Option<i64>) -> Result<(), LogicError> {
    let connector = etcd::get_connector(&session)?;
    connector.kv_put(
        key,
        value,
        ttl,
    ).await?;

    Ok(())
}

#[tauri::command]
pub async fn kv_put_with_lease(session: i32, key: String, value: Vec<u8>, lease: String) -> Result<(), LogicError> {
    let connector = etcd::get_connector(&session)?;
    let lease = i64::from_str(&lease).map_err(|e| {
        warn!("ttl parse error: {e}");
        LogicError::ArgumentError
    })?;
    connector.kv_put_with_lease(key, value, lease).await?;
    Ok(())
}

#[tauri::command]
pub async fn kv_delete(session: i32, keys: Vec<String>) -> Result<usize, LogicError> {
    let connector = etcd::get_connector(&session)?;
    let size = connector.kv_delete(keys).await?;
    Ok(size)
}

#[tauri::command]
pub async fn kv_get_history_versions(session: i32, key: String, start: i64, end: i64) -> Result<Vec<i64>, LogicError> {
    let connector = etcd::get_connector(&session)?;
    let versions = connector.kv_get_history_versions(key, start, end).await?;
    Ok(versions)
}