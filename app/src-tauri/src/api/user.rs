use crate::error::LogicError;
use crate::etcd;
use crate::transport::user::SerializableUser;

#[tauri::command]
pub async fn user_list(session: i32) -> Result<Vec<SerializableUser>, LogicError> {
    let lock = etcd::get_connector(&session)?;
    let mut connector = lock.lock().await;
    let users = connector.user_list().await?;
    Ok(users)
}

#[tauri::command]
pub async fn user_add(session: i32, user: String, password: String) -> Result<(), LogicError> {
    let lock = etcd::get_connector(&session)?;
    let mut connector = lock.lock().await;
    connector.user_add(user, password).await?;
    Ok(())
}

#[tauri::command]
pub async fn user_delete(session: i32, user: String) -> Result<(), LogicError> {
    let lock = etcd::get_connector(&session)?;
    let mut connector = lock.lock().await;
    connector.user_delete(user).await?;
    Ok(())
}

#[tauri::command]
pub async fn user_change_password(session: i32, user: String, new_password: String) -> Result<(), LogicError> {
    let lock = etcd::get_connector(&session)?;
    let mut connector = lock.lock().await;
    connector.user_change_password(user, new_password).await?;
    Ok(())
}

#[tauri::command]
pub async fn user_grant_role(session: i32, user: String, role: String) -> Result<(), LogicError> {
    let lock = etcd::get_connector(&session)?;
    let mut connector = lock.lock().await;
    connector.user_grant_role(user, role).await?;
    Ok(())
}

#[tauri::command]
pub async fn user_revoke_role(session: i32, user: String, role: String) -> Result<(), LogicError> {
    let lock = etcd::get_connector(&session)?;
    let mut connector = lock.lock().await;
    connector.user_revoke_role(user, role).await?;
    Ok(())
}

#[tauri::command]
pub async fn auth_enable(session: i32) -> Result<(), LogicError> {
    let lock = etcd::get_connector(&session)?;
    let mut connector = lock.lock().await;
    connector.auth_enable().await?;
    Ok(())
}

#[tauri::command]
pub async fn auth_disable(session: i32) -> Result<(), LogicError> {
    let lock = etcd::get_connector(&session)?;
    let mut connector = lock.lock().await;
    connector.auth_disable().await?;
    Ok(())
}