use std::io;
use std::string::FromUtf8Error;

use log::error;
use serde::{Serialize, Serializer};
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum LogicError {
    ConnectionLose,
    ArgError,
    EtcdClientError(etcd_client::Error),
    SshError(russh::Error),
    IoError(io::Error),
    SerdeError(serde_json::Error),
    Base64DecodeError(base64::DecodeError),
    ChannelRcvError(oneshot::error::RecvError),
    StringConvertError(FromUtf8Error),
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
            LogicError::SerdeError(e) => {
                error!("[Serde] {:?}", e);
                serializer.serialize_str(&e.to_string())
            }
            LogicError::Base64DecodeError(e) => {
                error!("[Base64Decode] {:?}", e);
                serializer.serialize_str(&e.to_string())
            }
            LogicError::ChannelRcvError(e) => {
                serializer.serialize_str(&e.to_string())
            }
            LogicError::StringConvertError(e) => {
                error!("[StrConvert] {:?}", e);
                serializer.serialize_str("Can not convert string with utf-8")
            }
            LogicError::ConnectionLose => serializer.serialize_str("connection lose"),
            LogicError::ArgError => serializer.serialize_str("invalid argument"),
        }
    }
}

impl From<etcd_client::Error> for LogicError {
    fn from(value: etcd_client::Error) -> Self {
        LogicError::EtcdClientError(value)
    }
}

impl From<russh::Error> for LogicError {
    fn from(value: russh::Error) -> Self {
        LogicError::SshError(value)
    }
}

impl From<io::Error> for LogicError {
    fn from(value: io::Error) -> Self {
        LogicError::IoError(value)
    }
}

impl From<serde_json::Error> for LogicError {
    fn from(value: serde_json::Error) -> Self {
        LogicError::SerdeError(value)
    }
}

impl From<base64::DecodeError> for LogicError {
    fn from(value: base64::DecodeError) -> Self {
        LogicError::Base64DecodeError(value)
    }
}

impl From<oneshot::error::RecvError> for LogicError {
    fn from(value: oneshot::error::RecvError) -> Self {
        LogicError::ChannelRcvError(value)
    }
}

impl From<FromUtf8Error> for LogicError {
    fn from(value: FromUtf8Error) -> Self {
        LogicError::StringConvertError(value)
    }
}