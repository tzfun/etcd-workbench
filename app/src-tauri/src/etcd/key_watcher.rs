use std::{
    collections::HashMap,
    ops::Deref,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use etcd_client::{EventType, WatchFilterType, WatchOptions};
use log::{debug, error, info, warn};
use tauri::{api::notification::Notification, AppHandle, Window};
use tokio::{
    sync::{oneshot, Mutex},
    task::JoinHandle,
    time::{sleep, Instant},
};

use crate::{
    error::LogicError, etcd::retry_key_watcher, transport::{
        connection::KeyMonitorConfig,
        event::{KeyWatchEvent, KeyWatchEventType},
        kv::SerializableKeyValue,
    }
};

use super::{
    etcd_connector::EtcdConnector, etcd_connector_handler::EtcdConnectorHandler,
    get_connector_optional,
};

struct KeyMonitorHolder {
    pub config: KeyMonitorConfig,
    pub task_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    pub shutdown_sender: Option<oneshot::Sender<i32>>,
    pub notify_flag: Arc<Mutex<u64>>,
}

impl KeyMonitorHolder {
    pub async fn shutdown(&mut self) {
        let shutdown_sender = self.shutdown_sender.take();
        if let Some(shutdown_sender) = shutdown_sender {
            if shutdown_sender.is_closed() {
                debug!("shutdown sender closed");
            } else {
                if let Err(e) = shutdown_sender.send(1) {
                    warn!("Failed to send shutdown signal to watcher: {}", e)
                } else {
                    debug!("Send shutdown signal successful");
                }
            }
        } else {
            debug!("shutdown sender not found");
        }
        let task_handle = Arc::clone(&self.task_handle);
        let mut task_handle = task_handle.lock().await;
        task_handle.take();
        drop(task_handle);
    }
}

pub struct KeyWatcher {
    session_id: i32,
    holder_map: HashMap<String, KeyMonitorHolder>,
    window: Window,
    handler: EtcdConnectorHandler,
}

impl KeyWatcher {
    pub fn new(session_id: i32, window: Window, handler: EtcdConnectorHandler) -> Self {
        Self {
            session_id,
            holder_map: HashMap::new(),
            window,
            handler,
        }
    }

    pub async fn remove_config_all(&mut self) {
        let all_keys: Vec<String> = self.holder_map.keys().map(|s| s.clone()).collect();

        for key in all_keys {
            self.remove_config(&key).await;
        }
    }

    /// 移除某一个配置
    pub async fn remove_config(&mut self, key: &String) {
        let mut holder = self.holder_map.remove(key);
        if let Some(holder) = holder.as_mut() {
            holder.shutdown().await;
            info!("Removed key watcher: {}", key);
        } else {
            warn!("Need not remove watcher because of not running: {}", key);
        }
    }

    pub async fn retry_config(&mut self, config: KeyMonitorConfig) -> Result<(), LogicError> {
        let mut retry_times = 1;
        while let Err(e) = self.set_config(config.clone()).await {
            warn!("Watcher retry failed {}: {:?}", retry_times, e);
            if retry_times == 10 {
                error!("Watcher retry attempts have exceeded {} times, retries will be discontinued.", retry_times);
                return Err(LogicError::ConnectionLose);
            }
            retry_times += 1;
            sleep(Duration::from_secs(3)).await;
        }
        
        Ok(())
    }

    /// 新增一个配置，在新增之前会移除已有的配置
    pub async fn set_config(&mut self, config: KeyMonitorConfig) -> Result<(), LogicError> {
        self.remove_config(&config.key).await;
        //  暂停的配置无需监听
        if config.paused {
            debug!("Key monitor is paused {}", config.key);
            return Ok(());
        }
        let mut options = WatchOptions::new().with_progress_notify().with_prev_key();
        if config.is_prefix {
            options = options.with_prefix();
        }

        let mut filters = vec![];
        if !config.monitor_remove {
            filters.push(WatchFilterType::NoDelete);
        }

        if !config.monitor_value_change && !config.monitor_create {
            filters.push(WatchFilterType::NoPut);
        }

        if !filters.is_empty() {
            options = options.with_filters(filters);
        }

        let connector = get_connector_optional(&self.session_id);
        if connector.is_none() {
            debug!("Connector is not exist: {}", config.key);
            return Ok(());
        }
        let mut connector = connector.unwrap();

        let (mut watcher, mut stream) = connector.watch(config.key.as_str(), Some(options)).await?;
        drop(connector);

        let task_handle = Arc::new(Mutex::new(None));
        let notify_flag = Arc::new(Mutex::new(0));

        let task_handle_clone = Arc::clone(&task_handle);
        let notify_flag_clone = Arc::clone(&notify_flag);
        let config_clone = config.clone();
        let window_clone = self.window.clone();
        let session_id = self.session_id;
        let key_clone = config.key.clone();

        let (shutdown_sender, shutdown_receiver) = oneshot::channel::<i32>();

        let task = tokio::spawn(async move {
            let watch_task = async {
                let mut need_retry = false;
                debug!("Started watcher: {}", config_clone.key);
                loop {
                    match stream.message().await {
                        Ok(Some(resp)) => {
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis() as u64;
                            for event in resp.events() {
                                match event.event_type() {
                                    EventType::Put => {
                                        if let Some(kv) = event.kv() {
                                            let watch_event = if kv.version() == 1 {
                                                //  create
                                                KeyWatchEvent {
                                                    session: session_id,
                                                    key: config_clone.key.clone(),
                                                    event_type: KeyWatchEventType::Create,
                                                    event_time: now,
                                                    prev_kv: None,
                                                    cur_kv: Some(SerializableKeyValue::from_ref(
                                                        kv,
                                                    )),
                                                }
                                            } else {
                                                //  modify
                                                KeyWatchEvent {
                                                    session: session_id,
                                                    key: config_clone.key.clone(),
                                                    event_type: KeyWatchEventType::Modify,
                                                    event_time: now,
                                                    prev_kv: event.prev_kv().map(|p_kv| {
                                                        SerializableKeyValue::from_ref(p_kv)
                                                    }),
                                                    cur_kv: Some(SerializableKeyValue::from_ref(
                                                        kv,
                                                    )),
                                                }
                                            };
                                            Self::on_event(
                                                &window_clone,
                                                watch_event,
                                                Arc::clone(&notify_flag_clone),
                                            ).await;
                                        }
                                    }
                                    EventType::Delete => {
                                        if let Some(kv) = event.kv() {
                                            let watch_event = KeyWatchEvent {
                                                session: session_id,
                                                key: config_clone.key.clone(),
                                                event_type: KeyWatchEventType::Remove,
                                                event_time: now,
                                                prev_kv: Some(SerializableKeyValue::from_ref(kv)),
                                                cur_kv: None,
                                            };

                                            Self::on_event(
                                                &window_clone,
                                                watch_event,
                                                Arc::clone(&notify_flag_clone),
                                            ).await;
                                        }
                                    }
                                }
                            }

                            if resp.canceled() {
                                info!(
                                    "Watcher {} canceled, reason: {}",
                                    watcher.watch_id(),
                                    resp.cancel_reason()
                                );
                                break;
                            }
                        }
                        Ok(None) => {
                            debug!("Got empty event response: {}", watcher.watch_id());
                        }
                        Err(e) => {
                            //  可能因为网络连接、Token超时等问题结束监听，需要重新恢复
                            warn!("Watcher error: {}", e);
                            need_retry = true;
                            break;
                        }
                    }
                }

                debug!("Watcher finished: {}", config_clone.key);
                need_retry
            };

            tokio::select! {
                v = shutdown_receiver => {
                    info!("Received watch shutdown: {}", v.unwrap_or(-1));
                    let _ = watcher.cancel().await;
                },
                retry = watch_task => {
                    if retry {
                        let _ = watcher.cancel().await;
                        //  重新监听
                        retry_key_watcher(session_id, config_clone);
                    }
                }
            }

            let mut lock = task_handle_clone.lock().await;
            *lock = None;
            drop(lock);
        });

        let mut lock = task_handle.lock().await;
        *lock = Some(task);
        drop(lock);

        self.holder_map.insert(
            (&config.key).clone(),
            KeyMonitorHolder {
                config,
                task_handle,
                shutdown_sender: Some(shutdown_sender),
                notify_flag,
            },
        );

        Ok(())
    }

    async fn on_event(window: &Window, event: KeyWatchEvent, notify_flag: Arc<Mutex<u64>>) {
        if !window.is_focused().unwrap() {
            let now = event.event_time;
            let mut lock = notify_flag.lock().await;
            //  3秒内不重复发送事件
            if now >= lock.deref() + 3000 {
                let res = Notification::new("com.beifengtz.etcdworkbench")
                    .title(event.event_type.desc())
                    .body(event.key.clone())
                    .show();
                if res.is_ok() {
                    *lock = now;
                }
            }
            drop(lock);
        }
        if let Err(e) = window.emit("key_watch_event", event) {
            error!("Failed to emit window event 'key_watch_event': {}", e);
        }
    }
}
