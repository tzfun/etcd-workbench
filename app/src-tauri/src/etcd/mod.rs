#![allow(unused)]
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use dashmap::mapref::one::RefMut;
use dashmap::DashMap;
use etcd_client::Error;
use lazy_static::lazy_static;
use tokio::sync::Mutex;
use crate::api::connection;
use crate::error::LogicError;
use crate::etcd::etcd_connector::EtcdConnector;
use crate::transport::connection::{Connection, ConnectionInfo, SessionData};

pub mod etcd_connector;
mod wrapped_etcd_client;
mod test;

static CONNECTION_ID_COUNTER: AtomicI32 = AtomicI32::new(1);

lazy_static! {
    static ref CONNECTION_POOL:DashMap<i32, EtcdConnector> = DashMap::with_capacity(2);
    static ref CONNECTION_INFO_POOL: DashMap<i32, ConnectionInfo> = DashMap::new();
}

fn gen_connection_id() -> i32 {
    CONNECTION_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

pub fn now_timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub async fn new_connector(name: String, connection: Connection) -> Result<SessionData, LogicError> {
    let user = if let Some(u) = &connection.user {
        Some(u.username.clone())
    } else {
        None
    };
    let namespace = connection.namespace.clone();
    let mut connector = EtcdConnector::new(connection).await?;
    connector.test_connection().await?;

    let root = if let Some(u) = &user {
        connector.user_is_root(u).await?
    } else {
        true
    };

    let connector_id = gen_connection_id();
    CONNECTION_POOL.insert(connector_id, connector);

    let info_result = connection::get_connection(name).await?;
    
    let mut connection_saved = false;
    let mut key_collection = None;
    let mut key_monitor_list = None;
    if let Some(info) = info_result {
        key_collection = Some((&info.key_collection).clone());
        key_monitor_list = Some((&info.key_monitor_list).clone());
        connection_saved = true;
        
        CONNECTION_INFO_POOL.insert(connector_id, info);
    }

    Ok(SessionData {
        id: connector_id,
        user,
        root,
        namespace,
        connection_saved,
        key_collection,
        key_monitor_list
    })
}

pub fn get_connector(id: &i32) -> Result<RefMut<'_, i32, EtcdConnector>, LogicError> {
    get_connector_optional(id).ok_or(LogicError::ConnectionLose)
}

pub fn get_connector_optional(id: &i32) -> Option<RefMut<'_, i32, EtcdConnector>> {
    CONNECTION_POOL.get_mut(id)
}

pub fn remove_connector(id: &i32) {
    if let Some(entry) = CONNECTION_POOL.remove(id) {
        drop(entry.1)
    }
}

pub fn get_connection_info_optional(id: &i32) -> Option<RefMut<'_, i32, ConnectionInfo>> {
    CONNECTION_INFO_POOL.get_mut(id)
}