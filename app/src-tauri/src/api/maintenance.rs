use crate::error::LogicError;
use crate::etcd;
use crate::transport::maintenance::SerializableCluster;

#[tauri::command]
pub async fn get_cluster(session: i32) -> Result<SerializableCluster, LogicError> {
    let connector = etcd::get_connector(&session)?;
    let cluster = connector.cluster_get().await?;
    Ok(cluster)
}