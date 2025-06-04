use serde::{Deserialize, Serialize};

use super::{connection::KeyMonitorConfig, kv::SerializableKeyValue};

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all="camelCase")]
pub enum DisconnectCase {
    SshChannelFailure,
    SshChannelEof,
    SshDisconnected(String),
    SshTunnelError(String),
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all="camelCase")]
pub struct SessionDisconnectedEvent {
    pub session_id: i32,
    pub case: DisconnectCase
}

#[repr(i8)]
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub enum KeyWatchEventType {
    Remove = 1,
    Create = 2,
    Modify = 3,
}

impl KeyWatchEventType {
    pub fn desc(&self) -> String {
        match self {
            Self::Remove => String::from("The key is removed"),
            Self::Create => String::from("The key is created"),
            Self::Modify => String::from("The key is modified"),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all="camelCase")]
pub struct KeyWatchEvent {
    pub session: i32,
    //  配置key值（全路径）
    pub key: String,
    //  事件触发的key值（全路径）
    pub event_key: String,
    pub event_type: KeyWatchEventType,
    pub event_time: u64,
    pub prev_kv: Option<SerializableKeyValue>,
    pub cur_kv: Option<SerializableKeyValue>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all="camelCase")]
pub struct KeyMonitorModifiedByServerEvent {
    pub session: i32,
    pub config: KeyMonitorConfig,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all="camelCase")]
pub struct UpdateDownloadingProgressEvent {
    pub chunk_length: usize,
    pub content_length: Option<u64>,
}