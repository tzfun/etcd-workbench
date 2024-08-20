use log::error;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

use dashmap::DashMap;
use lazy_static::lazy_static;
use tauri::Manager;
use tokio::sync::watch;
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
    tokio::spawn(async move {
        let (watch_sender, mut receiver) = watch::channel(SnapshotState::default());
        let app = Arc::new(app);
        let app_copy = Arc::clone(&app);
        tokio::spawn(async move {
            loop {
                {
                    let state = receiver.borrow_and_update();

                    let task = SNAPSHOT_TASK_POOL.get_mut(&task_id);
                    if let Some(mut t) = task {
                        t.state.replace(state.clone());

                        app.emit_all("snapshot_state", SnapshotStateInfo {
                            name: t.value().name.clone(),
                            id: task_id,
                            state: state.clone(),
                        }).unwrap();
                    }

                    if state.is_finished() {
                        print!("finished {:?}", state);
                        break;
                    }
                }

                if receiver.changed().await.is_err() {
                    error!("watch error");
                    break;
                }
            }
        });
        let res = connector.maintenance_snapshot(PathBuf::from(filepath), watch_sender).await;
        match res {
            Ok(task) => {
                SNAPSHOT_TASK_POOL.insert(id, task);
            }
            Err(e) => {
                error!("{:?}", e);
                app_copy.emit_all("notify_error", "").unwrap();
            }
        }
    });
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
        if let Some(state) = &entry.state {
            list.push(SnapshotStateInfo {
                name: entry.value().name.clone(),
                id: entry.key().clone(),
                state: state.clone(),
            });
        }
    }
    Ok(list)
}