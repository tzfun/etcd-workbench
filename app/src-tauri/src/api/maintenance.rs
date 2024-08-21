use std::path::PathBuf;
use std::sync::atomic::{AtomicI32, Ordering};

use dashmap::DashMap;
use lazy_static::lazy_static;
use tauri::Manager;
use tokio::sync::{mpsc, oneshot};
use crate::error::LogicError;
use crate::etcd;
use crate::etcd::etcd_connector::SnapshotTask;
use crate::transport::maintenance::{SerializableCluster, SnapshotState, SnapshotStateInfo};

#[allow(unused)]
static SNAPSHOT_TASK_ID_COUNTER: AtomicI32 = AtomicI32::new(1);
lazy_static! {
    static ref SNAPSHOT_TASK_POOL:DashMap<i32, SnapshotTask> = DashMap::with_capacity(1);
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
    filepath: String,
) -> Result<(), LogicError> {
    let connector = etcd::get_connector(&session)?;
    let id = SNAPSHOT_TASK_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

    let task_id = id.clone();
    let (watch_sender, mut receiver) = mpsc::channel::<(u64, u64, Option<String>)>(128);

    tokio::spawn(async move {

        while let Some(state) = receiver.recv().await {
            let task = SNAPSHOT_TASK_POOL.get_mut(&task_id);
            if let Some(mut t) = task {
                if let Some(err_msg) = &state.2 {
                    t.state.error_msg = Some(err_msg.clone())
                } else {
                    t.state.received += state.0;
                    t.state.remain = state.1;
                }

                let state = t.state.clone();

                app.emit_all("snapshot_state", SnapshotStateInfo {
                    name: t.value().name.clone(),
                    id: task_id,
                    state: state,
                }).unwrap();
            }
        }
    });
    let file_path = PathBuf::from(filepath);

    let file_name = if let Some(name) = file_path.file_name() {
        String::from_utf8_lossy(name.as_encoded_bytes()).to_string()
    } else {
        String::from("Snapshot Task")
    };

    let (stop_sender, stop_receiver) = oneshot::channel();

    let task = SnapshotTask {
        name: file_name,
        state: SnapshotState::default(),
        stop_notifier: Some(stop_sender),
    };
    SNAPSHOT_TASK_POOL.insert(id, task);

    connector.maintenance_snapshot(file_path, watch_sender, stop_receiver).await?;
    
    Ok(())
}

#[tauri::command]
pub fn maintenance_stop_snapshot_task(task_id: i32) -> Result<(), LogicError> {
    let task = SNAPSHOT_TASK_POOL.get_mut(&task_id);
    if let Some(mut t) = task {
        let stop_notifier = t.stop_notifier.take();
        if let Some(sender) = stop_notifier {
            sender.send(()).unwrap();
        }
        t.state.finished = true;
        t.state.error_msg = Some(String::from("Stopped"));
    }
    Ok(())
}

#[tauri::command]
pub fn maintenance_remove_snapshot_task(task_id: i32) -> Result<(), LogicError> {
    SNAPSHOT_TASK_POOL.remove(&task_id);
    Ok(())
}

#[tauri::command]
pub fn maintenance_list_snapshot_task() -> Result<Vec<SnapshotStateInfo>, LogicError> {
    let mut list = Vec::new();
    for entry in SNAPSHOT_TASK_POOL.iter() {
        list.push(SnapshotStateInfo {
            name: entry.value().name.clone(),
            id: entry.key().clone(),
            state: entry.state.clone(),
        });
    }
    Ok(list)
}