use std::sync::Arc;
use std::sync::atomic::{AtomicI32, AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use dashmap::DashMap;
use etcd_client::Error;
use lazy_static::lazy_static;

use crate::etcd::etcd_connector::EtcdConnector;
use crate::transport::connection::Connection;

pub mod etcd_connector;
mod test;

const CONNECTION_ID_COUNTER: AtomicI32 = AtomicI32::new(1);

lazy_static! {
    static ref CONNECTION_POOL:DashMap<i32, Arc<EtcdConnector>> = DashMap::with_capacity(2);
}

fn gen_connection_id() -> i32 {
    CONNECTION_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub fn now_timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub async fn new_connector(connection: Connection) -> Result<i32, Error> {
    let connector = EtcdConnector::new(connection).await?;
    let connector_id = gen_connection_id();
    CONNECTION_POOL.insert(connector_id, Arc::new(connector));
    Ok(connector_id)
}

pub fn get_connector(id: &i32) -> Option<Arc<EtcdConnector>> {
    let connector = CONNECTION_POOL.get(id)?;
    Some(Arc::clone(connector.value()))
}

pub fn remove_connector(id: &i32) {
    if let Some(entry) = CONNECTION_POOL.remove(id) {
        drop(entry.1)
    }
}