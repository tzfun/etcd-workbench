use crate::error::LogicError;
use crate::etcd;
use crate::transport::user::SerializablePermission;

#[tauri::command]
pub async fn role_list(session: i32) -> Result<Vec<String>, LogicError> {
    let connector = etcd::get_connector(&session)?;
    let roles = connector.role_list().await?;
    Ok(roles)
}
#[tauri::command]
pub async fn role_get_permissions(session: i32, role: String) -> Result<Vec<SerializablePermission>, LogicError> {
    let connector = etcd::get_connector(&session)?;
    let permissions = connector.role_get_permissions(role).await?;
    Ok(permissions)
}