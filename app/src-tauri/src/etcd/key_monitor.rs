use crate::etcd::etcd_connector::EtcdConnector;
use crate::transport::connection::KeyMonitorConfig;
use etcd_client::{GetOptions, GetResponse};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::AtomicI32;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::vec;
use tauri::api::notification::Notification;
use tauri::{Manager, Window};
use tokio::sync::Mutex;
use tokio::time::{interval, interval_at, Instant, MissedTickBehavior};

use super::get_connection_config;

#[repr(i8)]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum KeyMonitorEventType {
    Remove = 1,
    Create = 2,
    LeaseChange = 3,
    ValueChange = 4,
}

impl KeyMonitorEventType {
    pub fn desc(&self) -> String {
        match self {
            Self::Remove => String::from("removed"),
            Self::Create => String::from("created"),
            Self::LeaseChange => String::from("lease changed"),
            Self::ValueChange => String::from("value changed"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KeyMonitorEvent<T: Serialize + Clone> {
    pub session: i32,
    pub key: String,
    pub event_type: KeyMonitorEventType,
    pub event_time: u64,
    pub previous: Option<T>,
    pub current: Option<T>,
}

impl<T: Serialize + Clone> KeyMonitorEvent<T> {
    pub fn with(session: i32, key: String, event_type: KeyMonitorEventType) -> Self {
        Self {
            session,
            key,
            event_type,
            event_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            previous: None,
            current: None,
        }
    }

    pub fn with_value(
        session: i32,
        key: String,
        event_type: KeyMonitorEventType,
        previous: T,
        current: T,
    ) -> Self {
        Self {
            session,
            key,
            event_type,
            event_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            previous: Some(previous),
            current: Some(current),
        }
    }
}

#[derive(Clone)]
struct MonitorTask {
    config: KeyMonitorConfig,
    next_execute_time: Instant,
    first_run: bool,
    /// 前置查询出key是否存在
    previous_exist: Option<bool>,
    /// 前置查询出的value
    previous_value: Option<Vec<u8>>,
    /// 前置查询出的lease值
    previous_lease: Option<i64>,
}

impl MonitorTask {
    fn new(config: KeyMonitorConfig) -> Self {
        let interval = config.interval_seconds;
        Self {
            config,
            next_execute_time: Instant::now() + Duration::from_secs(interval),
            first_run: true,
            previous_exist: None,
            previous_value: None,
            previous_lease: None,
        }
    }

    async fn run(&mut self, session: i32, connector: &mut EtcdConnector, window: &Window) {
        let config = &self.config;
        debug!("Run monitor task: {}", config.key);

        let mut options = GetOptions::new().with_limit(1);
        if !config.monitor_value_change {
            options = options.with_keys_only();
        }

        let response = connector
            .kv_get_request(self.config.key.clone(), Some(options))
            .await;
        if let Err(ref e) = response {
            error!(
                "An exception occurred while executing the key monitoring task: {}",
                e
            );
            return;
        }
        let response = response.unwrap();
        if self.first_run {
            let exist = response.kvs().len() > 0;
            //  首次执行只存值，不判断
            if config.monitor_create || config.monitor_remove {
                self.previous_exist = Some(exist);
            }
            if exist {
                let kv = &response.kvs()[0];
                if config.monitor_lease_change {
                    self.previous_lease = Some(kv.lease());
                }
                if config.monitor_value_change {
                    self.previous_value = Some(kv.value().to_vec());
                }
            }
            self.first_run = false;
        } else {
            let exist = response.kvs().len() > 0;

            if config.monitor_create {
                if !self.previous_exist.unwrap_or(false) && exist {
                    debug!("Key create: {}", config.key);
                    self.on_event(
                        window,
                        KeyMonitorEvent::<i32>::with(
                            session,
                            config.key.clone(),
                            KeyMonitorEventType::Create,
                        ),
                    );
                }
            }

            if config.monitor_remove {
                if self.previous_exist.unwrap_or(false) && !exist {
                    debug!("Key removed: {}", config.key);
                    self.on_event(
                        window,
                        KeyMonitorEvent::<i32>::with(
                            session,
                            config.key.clone(),
                            KeyMonitorEventType::Remove,
                        ),
                    );
                }
            }

            self.previous_exist = Some(exist);

            if exist {
                let kv = &response.kvs()[0];
                if config.monitor_lease_change {
                    let previous_lease = self.previous_lease.unwrap_or(0);
                    let current_lease = kv.lease();
                    if previous_lease != current_lease {
                        debug!(
                            "Key lease changed: {}, {} -> {}",
                            config.key, previous_lease, current_lease
                        );
                        self.on_event(
                            window,
                            KeyMonitorEvent::<String>::with_value(
                                session,
                                config.key.clone(),
                                KeyMonitorEventType::LeaseChange,
                                previous_lease.to_string(),
                                current_lease.to_string(),
                            ),
                        );
                    }

                    self.previous_lease = Some(current_lease);
                }

                if config.monitor_value_change {
                    let previous_value = self.previous_value.clone().unwrap_or(vec![]);
                    let current_value = kv.value().to_vec();
                    if previous_value.ne(&current_value) {
                        debug!("Key value changed: {}", config.key);
                        self.on_event(
                            window,
                            KeyMonitorEvent::<Vec<u8>>::with_value(
                                session,
                                config.key.clone(),
                                KeyMonitorEventType::ValueChange,
                                previous_value,
                                current_value.clone(),
                            ),
                        );
                    }

                    self.previous_value = Some(current_value);
                }
            }
        }

        self.update_next_execute_time();
    }

    fn update_next_execute_time(&mut self) {
        self.next_execute_time = Instant::now() + Duration::from_secs(self.config.interval_seconds);
    }

    fn set_config(&mut self, config: KeyMonitorConfig) {
        self.config = config;
        self.first_run = true;
        self.update_next_execute_time();
    }

    fn on_event<T: Serialize + Clone>(&self, window: &Window, event: KeyMonitorEvent<T>) {
        if !window.is_focused().unwrap() {
            let _ = Notification::new("com.beifengtz.etcdworkbench")
                .title(format!("Key {}", event.event_type.desc()))
                .body(event.key.clone())
                .show();
        }
        window.emit("key_monitor", event);
    }
}

pub struct KeyMonitor {
    session_id: i32,
    etcd_connector: Option<EtcdConnector>,
    config_map: HashMap<String, MonitorTask>,
    running: bool,
    window: Window,
}

impl Drop for KeyMonitor {
    fn drop(&mut self) {
        debug!("drop key monitor: {}", self.session_id);
    }
}

impl KeyMonitor {
    pub fn new(session_id: i32, window: Window) -> Self {
        Self {
            session_id,
            etcd_connector: None,
            config_map: HashMap::new(),
            running: false,
            window,
        }
    }

    pub fn start(lock: Arc<Mutex<Self>>) {
        tokio::spawn(async move {
            {
                let lock = Arc::clone(&lock);
                let mut monitor = lock.lock().await;
                monitor.running = true;
            }
            info!("Key monitor started");
            let mut timer = interval_at(
                Instant::now() + Duration::from_secs(1),
                Duration::from_secs(1),
            );
            timer.set_missed_tick_behavior(MissedTickBehavior::Delay);
            loop {
                timer.tick().await;
                let mut monitor = lock.lock().await;

                if !monitor.running {
                    info!("Key monitor stopped. session: {}", monitor.session_id);
                    break;
                }

                if monitor.config_map.is_empty() {
                    info!("Because there is no available configuration, stop key monitoring. session: {}", monitor.session_id);
                    monitor.running = false;
                    break;
                }

                let now = Instant::now();
                let mut dirty_tasks = vec![];

                for (_, task) in monitor.config_map.iter() {
                    if now >= task.next_execute_time {
                        dirty_tasks.push(task.clone());
                    }
                }

                if !dirty_tasks.is_empty() {
                    let session_id = monitor.session_id;
                    let window = monitor.window.clone();

                    let connector = monitor.init_connector().await;
                    if connector.is_none() {
                        info!("Unable to initialize etcd connection, terminating monitor: {}", monitor.session_id);
                        monitor.running = false;
                        break;
                    }

                    let connector = connector.unwrap();
                    
                    for task in dirty_tasks.iter_mut() {
                        task.run(session_id, connector, &window).await;
                    }

                    for task in dirty_tasks {
                        monitor.config_map.insert(task.config.key.clone(), task);
                    }
                }
            }
            drop(timer);
        });
    }

    pub async fn stop(lock: Arc<Mutex<Self>>) {
        let mut monitor = lock.lock().await;
        monitor.running = false;
        info!("Key monitor stopped: {}", monitor.session_id);
    }

    pub async fn set_config(lock: Arc<Mutex<Self>>, config: KeyMonitorConfig) {
        info!("Set key monitor: {}", config.key);
        let mut monitor = lock.lock().await;
        let exist_task = monitor.config_map.get_mut(&config.key);
        if let Some(task) = exist_task {
            task.set_config(config);
        } else {
            monitor
                .config_map
                .insert((&config.key).clone(), MonitorTask::new(config));

            //  冷启动
            if !monitor.running {
                drop(monitor);
                KeyMonitor::start(lock);
            }
        }
    }

    pub async fn remove_config(lock: Arc<Mutex<Self>>, key: &String) {
        let mut monitor = lock.lock().await;
        monitor.config_map.remove(key);
        info!("Removed key monitor: {}", key);
    }

    pub fn add_config(&mut self, config: KeyMonitorConfig) {
        self.config_map
            .insert((&config.key).clone(), MonitorTask::new(config));
    }

    async fn init_connector(&mut self) -> Option<&mut EtcdConnector> {
        let not_init = self.etcd_connector.is_none();
        if not_init {
            let session_id = self.session_id;
            let config = get_connection_config(&session_id);
            if let Some(connection) = config {
                let result = EtcdConnector::new(connection.value().clone()).await;
                match result {
                    Ok(connector) => {
                        self.etcd_connector = Some(connector);
                    },
                    Err(e) => {
                        error!("Failed to init connector: {:?}", e);
                        return None
                    }
                }
            } else {
                return None
            }
        }

        return self.etcd_connector.as_mut()
    }
 }
