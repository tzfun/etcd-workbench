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
pub async fn role_add(session: i32, role: String) -> Result<(), LogicError> {
    let connector = etcd::get_connector(&session)?;
    connector.role_add(role).await?;
    Ok(())
}

#[tauri::command]
pub async fn role_delete(session: i32, role: String) -> Result<(), LogicError> {
    let connector = etcd::get_connector(&session)?;
    connector.role_delete(role).await?;
    Ok(())
}

#[tauri::command]
pub async fn role_get_permissions(session: i32, role: String) -> Result<Vec<SerializablePermission>, LogicError> {
    let connector = etcd::get_connector(&session)?;
    let permissions = connector.role_get_permissions(role).await?;
    Ok(permissions)
}

#[tauri::command]
pub async fn role_grant_permission(session: i32, role: String, permission: SerializablePermission) -> Result<(), LogicError> {
    let connector = etcd::get_connector(&session)?;
    connector.role_grant_permission(role, permission).await?;
    Ok(())
}

#[tauri::command]
pub async fn role_revoke_permission(session: i32, role: String, permission: SerializablePermission) -> Result<(), LogicError> {
    let connector = etcd::get_connector(&session)?;
    connector.role_revoke_permission(role, permission).await?;
    Ok(())
}