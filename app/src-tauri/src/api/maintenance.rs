use std::path::PathBuf;
use std::sync::atomic::{AtomicI32, Ordering};

use dashmap::DashMap;
use lazy_static::lazy_static;
use log::error;
use tauri::Manager;
use tokio::task::JoinHandle;
use crate::error::LogicError;
use crate::etcd;
use crate::transport::maintenance::{SerializableCluster, SnapshotStateEvent};

#[allow(unused)]
static SNAPSHOT_TASK_ID_COUNTER: AtomicI32 = AtomicI32::new(1);
lazy_static! {
    static ref SNAPSHOT_TASK_POOL:DashMap<i32, JoinHandle<()>> = DashMap::with_capacity(1);
}

#[tauri::command]
pub async fn get_cluster(session: i32) -> Result<SerializableCluster, LogicError> {
    let connector = etcd::get_connector(&session)?;
    let cluster = connector.cluster_get().await?;
    Ok(cluster)
}


#[tauri::command]
pub async fn maintenance_defragment(session: i32) -> Result<(), LogicError> {
    let connector = etcd::get_connector(&session)?;
    connector.maintenance_defragment().await?;
    Ok(())
}

#[tauri::command]
pub async fn maintenance_create_snapshot_task(
    app: tauri::AppHandle,
    session: i32,
    dir: String,
) -> Result<i32, LogicError> {
    let connector = etcd::get_connector(&session)?;
    let id = SNAPSHOT_TASK_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

    let event_id = id.clone();
    let mut receiver = connector.maintenance_snapshot(PathBuf::from(dir)).await?;
    let handle = tokio::spawn(async move {
        loop {
            let res = receiver.changed().await;

            if let Ok(_) = res {
                let state = receiver.borrow();
                let state_owned = state.clone();
                app.emit_all("snapshot_state", SnapshotStateEvent {
                    id: event_id,
                    state:state_owned,
                }).unwrap();

                if !state.success || state.remain <= 0 {
                    break
                }
            } else {
                error!("e");
                break
            }
        }
    });

    SNAPSHOT_TASK_POOL.insert(id, handle);

    Ok(id)
}