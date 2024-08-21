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
    pub finished: bool,
    pub received: u64,
    pub remain: u64,
    pub error_msg: Option<String>
}

impl Clone for SnapshotState {
    fn clone(&self) -> Self {
        Self {
            finished: self.finished,
            received: self.received,
            remain: self.remain,
            error_msg: self.error_msg.clone()
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