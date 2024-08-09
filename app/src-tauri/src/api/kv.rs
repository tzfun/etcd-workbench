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
pub async fn kv_get_kv(session: i32, key: String) -> Result<SerializableKeyValue, LogicError> {
    let connector = etcd::get_connector(&session)?;
    let kv = connector.kv_get(key).await?;
    Ok(kv)
}