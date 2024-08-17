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
    pub remain: u64,
    pub msg: Option<String>
}

impl SnapshotState {
    pub fn success(remain: u64) -> Self {
        Self {
            success: true,
            remain,
            msg: None
        }
    }

    pub fn failed(msg: String) -> Self {
        Self {
            success: false,
            remain: 064,
            msg: Some(msg)
        }
    }
}

impl Clone for SnapshotState {
    fn clone(&self) -> Self {
        Self {
            success: self.success,
            remain: self.remain,
            msg: self.msg.clone()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all="camelCase")]
pub struct SnapshotStateEvent {
    pub id: i32,
    pub state: SnapshotState
}