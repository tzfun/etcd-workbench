use std::str::FromStr;

use log::warn;

use crate::error::LogicError;
use crate::etcd;
use crate::transport::kv::{SerializableLeaseInfo};

#[tauri::command]
pub async fn leases(session: i32) -> Result<Vec<String>, LogicError> {
    let connector = etcd::get_connector(&session)?;
    let leases = connector.leases().await?;
    Ok(leases)
}

#[tauri::command]
pub async fn lease_get(session: i32, lease: String) -> Result<SerializableLeaseInfo, LogicError> {
    let connector = etcd::get_connector(&session)?;
    let lease = i64::from_str(&lease).map_err(|e| {
        warn!("lease parse error: {e}");
        LogicError::ArgumentError
    })?;
    let info = connector.lease_get(lease).await?;
    Ok(info)
}