use std::string::String;
use std::io;
use std::string::FromUtf8Error;

use log::error;
use serde::{Deserialize, Serialize, Serializer};
use tokio::sync::oneshot;

use crate::utils::aes_util::AesError;

#[derive(Debug, Serialize, Deserialize)]
enum ErrorType {
    /// 身份认证失效，需要重新连接
    Unauthenticated,
    /// etcd客户端异常
    EtcdClientError,
    /// ssh隧道异常
    SshClientError,
    /// ssh隧道异常
    SshKeysError,
    /// 应用异常，一般是代码级的错误
    AppError,
    /// 参数错误
    ArgumentError,
    /// 资源不存在
    ResourceNotExist,
    /// 权限被拒绝
    PermissionDenied,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct ErrorPayload<'a> {
    err_type: ErrorType,
    err_msg: &'a str,
}

#[derive(Debug)]
pub enum LogicError<> {
    MsgError(String),
    ConnectionLose,
    ArgumentError,
    ResourceNotExist(&'static str),
    EtcdClientError(etcd_client::Error),
    SshError(russh::Error),
    SshKeysError(russh::keys::Error),
    IoError(io::Error),
    SerdeError(serde_json::Error),
    AesError(AesError),
    ChannelRcvError(oneshot::error::RecvError),
    StringConvertError(FromUtf8Error)
}

impl Serialize for LogicError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            LogicError::MsgError(e) => {
                ErrorPayload {
                    err_type: ErrorType::ResourceNotExist,
                    err_msg: e.as_str(),
                }.serialize(serializer)
            },
            LogicError::EtcdClientError(e) => {
                error!("[ETCD] {:?}", e);
                match e {
                    etcd_client::Error::GRpcStatus(status) => {
                        let code = status.code();
                        let msg = status.message();
                        let code = code as i32;

                        let msg = if msg.starts_with("etcdserver:") {
                            msg.replace("etcdserver:", "")
                        } else {
                            String::from(msg)
                        };

                        let msg = msg.as_str();

                        if code == 16 { //  Unauthenticated
                            ErrorPayload {
                                err_type: ErrorType::Unauthenticated,
                                err_msg: msg,
                            }.serialize(serializer)
                        } else if code == 7 {   //  PermissionDenied
                            ErrorPayload {
                                err_type: ErrorType::PermissionDenied,
                                err_msg: msg,
                            }.serialize(serializer)
                        } else {
                            ErrorPayload {
                                err_type: ErrorType::EtcdClientError,
                                err_msg: msg,
                            }.serialize(serializer)
                        }
                    }
                    etcd_client::Error::InvalidArgs(msg) => {
                        ErrorPayload {
                            err_type: ErrorType::EtcdClientError,
                            err_msg: msg,
                        }.serialize(serializer)
                    }
                    _ => {
                        let msg = e.to_string();
                        ErrorPayload {
                            err_type: ErrorType::EtcdClientError,
                            err_msg: msg.as_str(),
                        }.serialize(serializer)
                    }
                }
            }
            LogicError::SshError(e) => {
                error!("[SSH] {:?}", e);
                let msg = e.to_string();
                ErrorPayload {
                    err_type: ErrorType::SshClientError,
                    err_msg: msg.as_str(),
                }.serialize(serializer)
            }
            LogicError::SshKeysError(e) => {
                error!("[SSH Key] {:?}", e);
                let msg = e.to_string();
                ErrorPayload {
                    err_type: ErrorType::SshKeysError,
                    err_msg: msg.as_str(),
                }.serialize(serializer)
            }
            LogicError::IoError(e) => {
                error!("[IO] {:?}", e);
                let msg = e.to_string();
                ErrorPayload {
                    err_type: ErrorType::AppError,
                    err_msg: msg.as_str(),
                }.serialize(serializer)
            }
            LogicError::SerdeError(e) => {
                error!("[Serde] {:?}", e);
                let msg = e.to_string();
                ErrorPayload {
                    err_type: ErrorType::AppError,
                    err_msg: msg.as_str(),
                }.serialize(serializer)
            }
            LogicError::AesError(e) => {
                error!("[AesError] {:?}", e);
                let msg = e.to_string();
                ErrorPayload {
                    err_type: ErrorType::AppError,
                    err_msg: msg.as_str(),
                }.serialize(serializer)
            }
            LogicError::ChannelRcvError(e) => {
                let msg = e.to_string();
                ErrorPayload {
                    err_type: ErrorType::AppError,
                    err_msg: msg.as_str(),
                }.serialize(serializer)
            }
            LogicError::StringConvertError(e) => {
                error!("[StrConvert] {:?}", e);
                ErrorPayload {
                    err_type: ErrorType::AppError,
                    err_msg: "Can not convert string with utf-8",
                }.serialize(serializer)
            }
            LogicError::ConnectionLose => {
                ErrorPayload {
                    err_type: ErrorType::Unauthenticated,
                    err_msg: "connection lose",
                }.serialize(serializer)
            }
            LogicError::ArgumentError => {
                ErrorPayload {
                    err_type: ErrorType::ArgumentError,
                    err_msg: "invalid argument",
                }.serialize(serializer)
            }
            LogicError::ResourceNotExist(e) => {
                ErrorPayload {
                    err_type: ErrorType::ResourceNotExist,
                    err_msg: e,
                }.serialize(serializer)
            }
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

impl From<russh::keys::Error> for LogicError {
    fn from(value: russh::keys::Error) -> Self {
        LogicError::SshKeysError(value)
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

impl From<AesError> for LogicError {
    fn from(value: AesError) -> Self {
        LogicError::AesError(value)
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