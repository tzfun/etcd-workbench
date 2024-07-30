use crate::api::LogicError;
use crate::transport::kv::SerializableKeyValue;

#[tauri::command]
pub async fn kv_list() -> Result<Vec<SerializableKeyValue>, LogicError> {

}