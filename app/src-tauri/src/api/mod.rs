use serde::{Serialize, Serializer};

pub mod kv;
pub mod user;
pub mod maintenance;
pub mod connection;

#[derive(Debug)]
pub enum LogicError {
    EtcdClientError(etcd_client::Error),
    ConnectionLose
}
impl Serialize for LogicError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            LogicError::EtcdClientError(e) => {
                serializer.serialize_str(&e.to_string())
            },
            LogicError::ConnectionLose => serializer.serialize_str("connection lose")
        }
    }
}

impl From<etcd_client::Error> for LogicError {
    fn from(value: etcd_client::Error) -> Self {
        LogicError::EtcdClientError(value)
    }
}