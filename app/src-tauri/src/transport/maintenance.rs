use std::str::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct SerializableCluster {
    pub id: String,
    pub member_id: String,
    pub revision: i64,
    pub members: Vec<SerializableClusterMember>,
    pub status: SerializableClusterStatus
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct SerializableClusterMember {
    pub id: String,
    pub name: String,
    pub peer_uri: Vec<String>,
    pub client_uri: Vec<String>,
    pub alarm_type: i32
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct SerializableClusterStatus {
    pub version: String,
    pub db_size_allocated: i64,
    pub db_size_used: i64,
    pub leader: String,
    pub raft_index: String,
    pub raft_term: String,
    pub raft_applied_index: String,
    pub errors: Vec<String>
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all="camelCase")]
pub struct SnapshotState {
    pub success: bool,
    pub remain: String,
    pub msg: Option<String>
}

impl SnapshotState {
    pub fn success(remain: u64) -> Self {
        Self {
            success: true,
            remain: remain.to_string(),
            msg: None
        }
    }

    pub fn failed(msg: String) -> Self {
        Self {
            success: false,
            remain: String::from("0"),
            msg: Some(msg)
        }
    }

    pub fn get_remain(&self) -> u64 {
        u64::from_str(&self.remain.as_str()).unwrap()
    }

    pub fn is_finished(&self) -> bool {
        !self.success || self.get_remain() == 0
    }
}

impl Clone for SnapshotState {
    fn clone(&self) -> Self {
        Self {
            success: self.success,
            remain: self.remain.clone(),
            msg: self.msg.clone()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all="camelCase")]
pub struct SnapshotStateInfo {
    pub name: String,
    pub id: i32,
    pub state: SnapshotState
}