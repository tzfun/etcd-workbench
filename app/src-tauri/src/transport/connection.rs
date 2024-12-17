use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectionUser {
    pub username: String,
    pub password: String,
}

pub type TlsCertificate = Vec<u8>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TlsIdentity {
    pub cert: TlsCertificate,
    pub key: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectionTls {
    pub domain: Option<String>,
    pub cert: Vec<TlsCertificate>,
    pub identity: Option<TlsIdentity>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all="camelCase")]
pub struct SshPrivateKey {
    pub key: Vec<u8>,
    pub passphrase: Option<String>,
    /// ssh_key::algorithm::HashAlg
    #[serde(default = "default_private_key_hash_alg")]
    pub hash_algorithm: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SshIdentity {
    pub password: Option<String>,
    pub key: Option<SshPrivateKey>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectionSsh {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub identity: Option<SshIdentity>,
}

/// 连接必要数据
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Connection {
    pub host: String,
    pub port: u16,
    pub namespace: Option<String>,
    pub user: Option<ConnectionUser>,
    pub tls: Option<ConnectionTls>,
    pub ssh: Option<ConnectionSsh>,
}

/// 连接信息
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all="camelCase")]
pub struct ConnectionInfo {
    //  连接名
    pub name: String,
    pub connection: Connection,
    //  key收藏夹(全路径)
    #[serde(default = "default_key_collection")]
    pub key_collection: Vec<String>,
    //  key监控列表
    #[serde(default = "default_key_monitor_list")]
    pub key_monitor_list: Vec<KeyMonitorConfig>
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct SessionData {
    pub id: i32,
    pub user: Option<String>,
    pub root: bool,
    pub connection_saved: bool,
    pub namespace: Option<String>,
    pub key_collection: Option<Vec<String>>,
    pub key_monitor_list: Option<Vec<KeyMonitorConfig>>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all="camelCase")]
pub struct KeyMonitorConfig {
    //  key值（全路径）
    pub key: String,
    pub interval_seconds: u64,
    pub monitor_lease_change: bool,
    pub monitor_value_change: bool,
    pub monitor_create: bool,
    pub monitor_remove: bool,
}

impl KeyMonitorConfig {
    pub fn merge(&mut self, other: &KeyMonitorConfig) {
        self.interval_seconds = other.interval_seconds;
        self.monitor_lease_change = other.monitor_lease_change;
        self.monitor_value_change = other.monitor_value_change;
        self.monitor_create = other.monitor_create;
        self.monitor_remove = other.monitor_remove;
    }
}

fn default_private_key_hash_alg() -> Option<String> {
    None
}

fn default_key_collection() -> Vec<String> {
    vec![]
}

fn default_key_monitor_list() -> Vec<KeyMonitorConfig> {
    vec![]
}