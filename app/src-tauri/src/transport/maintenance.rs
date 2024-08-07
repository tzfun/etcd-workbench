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