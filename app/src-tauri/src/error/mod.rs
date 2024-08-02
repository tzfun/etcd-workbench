use std::io;
use log::error;
use serde::{Serialize, Serializer};

#[derive(Debug)]
pub enum LogicError {
    EtcdClientError(etcd_client::Error),
    SshError(ssh2::Error),
    IoError(io::Error),
    ConnectionLose,
}

impl Serialize for LogicError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            LogicError::EtcdClientError(e) => {
                error!("[ETCD] {:?}", e);
                match e {
                    etcd_client::Error::GRpcStatus(status) => {
                        serializer.serialize_str(status.code().description())
                    }
                    _ => {
                        serializer.serialize_str(&e.to_string())
                    }
                }
            }
            LogicError::SshError(e) => {
                error!("[SSH] {:?}", e);
                serializer.serialize_str(&e.to_string())
            }
            LogicError::IoError(e) => {
                error!("[IO] {:?}", e);
                serializer.serialize_str(&e.to_string())
            }
            LogicError::ConnectionLose => serializer.serialize_str("connection lose")
        }
    }
}

impl From<etcd_client::Error> for LogicError {
    fn from(value: etcd_client::Error) -> Self {
        LogicError::EtcdClientError(value)
    }
}

impl From<ssh2::Error> for LogicError {
    fn from(value: ssh2::Error) -> Self {
        LogicError::SshError(value)
    }
}

impl From<io::Error> for LogicError {
    fn from(value: io::Error) -> Self {
        LogicError::IoError(value)
    }
}