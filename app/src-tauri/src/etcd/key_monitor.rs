use crate::etcd::etcd_connector::EtcdConnector;
use crate::etcd::get_connector_optional;
use crate::transport::connection::KeyMonitorConfig;
use etcd_client::{GetOptions, GetResponse};
use log::{debug, error, info};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use std::vec;
use tokio::sync::Mutex;
use tokio::time::{interval, interval_at, Instant, MissedTickBehavior};

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

    async fn run(&mut self, connector: &mut EtcdConnector) {
        let response = connector
            .kv_get_request(self.config.key.clone(), None)
            .await;
        if let Err(ref e) = response {
            error!(
                "An exception occurred while executing the key monitoring task: {}",
                e
            );
            return;
        }
        let response = response.unwrap();
        let config = &self.config;
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
                    //  TODO 通知创建事件
                    debug!("Key create: {}", config.key);
                }

            }

            if config.monitor_remove {
                if self.previous_exist.unwrap_or(false) && !exist {
                    //  TODO 通知移除事件
                    debug!("Key removed: {}", config.key);
                }
            }

            self.previous_exist = Some(exist);

            if exist {
                let kv = &response.kvs()[0];
                if config.monitor_lease_change {
                    let previous_lease = self.previous_lease.unwrap_or(0);
                    let current_lease = kv.lease();
                    if previous_lease != current_lease {
                        //  TODO 通知Lease变更事件
                        debug!("Key lease changed: {}, {} -> {}", config.key, previous_lease, current_lease);
                    }

                    self.previous_lease = Some(current_lease);
                }

                if config.monitor_value_change {
                    let previous_value = self.previous_value.as_ref().unwrap();
                    let current_value = kv.value().to_vec();
                    if previous_value.ne(&current_value) {
                        //  TODO 通知值变更事件
                    debug!("Key value changed: {}", config.key);
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
}

pub struct KeyMonitor {
    session_id: i32,
    config_map: HashMap<String, MonitorTask>,
    running: bool,
}

impl KeyMonitor {
    pub fn new(session_id: i32) -> Self {
        Self {
            session_id,
            config_map: HashMap::new(),
            running: false,
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
                
                let session_id = monitor.session_id;
                let now = Instant::now();
                let mut tasks = vec![];

                for (key, task) in monitor.config_map.iter_mut() {
                    if now >= task.next_execute_time {
                        tasks.push(task);
                    }
                }

                if !tasks.is_empty() {
                    let connector_op = get_connector_optional(&session_id);
                    
                    if connector_op.is_none() {
                        info!("Connector not found, key monitor will be stopped. session: {}", session_id);
                        monitor.running = false;
                        break;
                    }
                    let mut connector = connector_op.unwrap();
                    for task in tasks {
                        task.run(&mut connector).await;
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
        self.config_map.insert((&config.key).clone(), MonitorTask::new(config));
    }
}
