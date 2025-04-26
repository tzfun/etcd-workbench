use crate::error::LogicError;
use crate::etcd;
use crate::transport::kv::{KVPutResult, SearchResult, SerializableKeyValue};
use log::warn;
use std::str::FromStr;

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
