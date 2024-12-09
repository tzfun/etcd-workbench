use std::str::FromStr;

use log::warn;

use crate::error::LogicError;
use crate::etcd;
use crate::transport::kv::SerializableLeaseInfo;

#[tauri::command]
pub async fn leases(session: i32) -> Result<Vec<String>, LogicError> {
    let lock = etcd::get_connector(&session)?;
    let mut connector = lock.lock().await;
    let leases = connector.leases().await?;
    Ok(leases)
}

#[tauri::command]
pub async fn lease_get(session: i32, lease: String) -> Result<SerializableLeaseInfo, LogicError> {
    let lock = etcd::get_connector(&session)?;
    let mut connector = lock.lock().await;
    let lease = i64::from_str(&lease).map_err(|e| {
        warn!("lease parse error: {e}");
        LogicError::ArgumentError
    })?;
    let info = connector.lease_get(lease).await?;
    Ok(info)
}

#[tauri::command]
pub async fn lease_grant(session: i32, ttl: i64, lease: Option<String>) -> Result<String, LogicError> {
    let lock = etcd::get_connector(&session)?;
    let mut connector = lock.lock().await;

    let lease = if let Some(s) = lease {
        Some(i64::from_str(&s).map_err(|e| {
            warn!("lease parse error: {e}");
            LogicError::ArgumentError
        })?)
    } else {
        None
    };

    let lease_id = connector.lease_grant(ttl, lease).await?;
    Ok(lease_id.to_string())
}

#[tauri::command]
pub async fn lease_revoke(session: i32, lease: String) -> Result<(), LogicError> {
    let lock = etcd::get_connector(&session)?;
    let mut connector = lock.lock().await;
    let lease = i64::from_str(&lease).map_err(|e| {
        warn!("lease parse error: {e}");
        LogicError::ArgumentError
    })?;
    connector.lease_revoke(lease).await?;
    Ok(())
}