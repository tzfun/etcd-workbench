use crate::api::LogicError;
use crate::etcd;
use crate::etcd::etcd_connector::EtcdConnector;
use crate::transport::connection::Connection;

#[tauri::command]
pub async fn connect_test(connection: Connection) -> Result<(), LogicError> {
    let connector = EtcdConnector::new(connection).await?;
    connector.user_is_root()
}

#[tauri::command]
pub async fn connect(connection: Connection) -> Result<i32, LogicError> {

}

#[tauri::command]
pub async fn disconnect(session: i32) -> Result<(), LogicError> {
    etcd::remove_connector(&session);
    Ok(())
}