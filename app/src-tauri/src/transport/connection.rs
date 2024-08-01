use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionUser {
    pub username: String,
    pub password: String,
}

pub type TlsCertificate = Vec<u8>;

#[derive(Debug, Serialize, Deserialize)]
pub struct TlsIdentity {
    pub cert: TlsCertificate,
    pub key: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionTls {
    pub domain: Option<String>,
    pub cert: Vec<TlsCertificate>,
    pub identity: Option<TlsIdentity>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SshPrivateKey {
    pub key: Vec<u8>,
    pub passphrase: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SshIdentity {
    pub password: Option<String>,
    pub key: Option<SshPrivateKey>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionSsh {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub identity: Option<SshIdentity>,
}

/// 连接必要数据
#[derive(Debug, Serialize, Deserialize)]
pub struct Connection {
    pub host: String,
    pub port: u16,
    pub namespace: Option<String>,
    pub user: Option<ConnectionUser>,
    pub tls: Option<ConnectionTls>,
    pub ssh: Option<ConnectionSsh>,
}

/// 连接信息
#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionInfo {
    //  连接名
    pub name: String,
    pub connection: Connection
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionData {
    pub id: i32,
    pub user: Option<String>,
    pub root: bool
}