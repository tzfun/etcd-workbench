use crate::api::LogicError;
use crate::etcd;
use crate::etcd::etcd_connector::EtcdConnector;
use crate::transport::connection::{Connection, SessionData};

#[tauri::command]
pub async fn connect_test(connection: Connection) -> Result<(), LogicError> {
    let connector = EtcdConnector::new(connection).await?;
    connector.test_connection().await?;
    Ok(())
}

#[tauri::command]
pub async fn connect(connection: Connection) -> Result<SessionData, LogicError> {
    let session = etcd::new_connector(connection).await?;
    Ok(session)
}

#[tauri::command]
pub async fn disconnect(session: i32) -> Result<(), LogicError> {
    etcd::remove_connector(&session);
    Ok(())
}