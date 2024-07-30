use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializableCluster {
    pub id: String,
    pub revision: i64,
    pub raft_term: String,
    pub members: Vec<SerializableClusterMember>,
    pub status: SerializableClusterStatus
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializableClusterMember {
    pub id: String,
    pub name: String,
    pub peer_uri: Vec<String>,
    pub client_uri: Vec<String>,
    pub alarm_type: i32
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializableClusterStatus {
    pub version: String,
    pub db_size: i64,
    pub raft_used_db_size: i64,
    pub leader: String,
    pub raft_index: String,
    pub raft_term: String,
    pub raft_applied_index: String,
    pub errors: Vec<String>
}