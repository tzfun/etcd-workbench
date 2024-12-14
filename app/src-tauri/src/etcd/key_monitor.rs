use crate::etcd::etcd_connector::EtcdConnector;
use crate::etcd::get_connector_optional;
use crate::transport::connection::KeyMonitorConfig;
use etcd_client::{GetOptions, GetResponse};
use log::{error, info};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::{interval, interval_at, Instant};

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
                }
            }

            if config.monitor_remove {
                if self.previous_exist.unwrap_or(false) && !exist {
                    //  TODO 通知移除事件
                }
            }

            if exist {
                let kv = &response.kvs()[0];
                if config.monitor_lease_change {
                    let previous_lease = self.previous_lease.unwrap_or(0);
                    let current_lease = kv.lease();
                    if previous_lease != current_lease {
                        //  TODO 通知Lease变更事件
                    }
                }

                if config.monitor_value_change {
                    let previous_value = self.previous_value.as_ref().unwrap_or(&vec![]);
                    let current_value = response.kvs().to_vec();
                    if previous_value.ne(&current_value) {
                        //  TODO 通知值变更事件
                    }
                }
            }
        }

        self.update_next_execute_time();
    }

    fn update_next_execute_time(&mut self) {
        self.next_execute_time = Instant::now() + Duration::from_secs(self.config.interval_seconds);
    }

    fn update_config(&mut self, config: KeyMonitorConfig) {
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
            running: true,
        }
    }
    pub fn start(self: Arc<Mutex<Self>>) {
        tokio::spawn(async move {
            let mut interval = interval_at(
                Instant::now() + Duration::from_secs(1),
                Duration::from_secs(1),
            );
            loop {
                interval.tick().await;

                let mut monitor = self.lock().await;

                if !monitor.running {
                    break;
                }

                let connector_op = get_connector_optional(&monitor.session_id);
                if connector_op.is_none() {
                    break;
                }
                let mut connector = connector_op.unwrap();
                let now = Instant::now();
                let mut tasks = vec![];

                for (key, task) in monitor.config_map.iter_mut() {
                    if now >= task.next_execute_time {
                        tasks.push(task);
                    }
                }

                if !tasks.is_empty() {
                    for task in tasks {
                        task.run(&mut connector);
                    }
                }
                drop(connector);
            }

            info!("Key monitor stopped. session: {}", self.session_id);
        });
    }

    pub async fn stop(self: Arc<Mutex<Self>>) {
        let mut monitor = self.lock().await;
        monitor.running = false;
    }

    pub async fn set_config(self: Arc<Mutex<Self>>, config: KeyMonitorConfig) {
        let monitor = self.lock().await;
        let exist_task = monitor.config_map.get_mut(&config.key);
        if let Some(task) = exist_task {
            task.update_config(config);
        } else {
            monitor
                .config_map
                .insert((&config.key).clone(), MonitorTask::new(config));
        }
        drop(monitor);
    }
}
