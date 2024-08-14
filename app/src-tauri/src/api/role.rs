use crate::error::LogicError;
use crate::etcd;

#[tauri::command]
pub async fn role_list(session: i32) -> Result<Vec<String>, LogicError> {
    let connector = etcd::get_connector(&session)?;
    let roles = connector.role_list().await?;
    Ok(roles)
}