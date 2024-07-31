use crate::api::LogicError;
use crate::etcd;
use crate::transport::kv::SerializableKeyValue;

#[tauri::command]
pub async fn kv_get_all_keys(session: i32) -> Result<Vec<SerializableKeyValue>, LogicError> {
    let connector = etcd::get_connector(&session).ok_or(LogicError::ConnectionLose)?;
    let keys = connector.kv_get_all_keys().await?;
    Ok(keys)
}