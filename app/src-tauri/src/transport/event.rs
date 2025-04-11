use serde::{Deserialize, Serialize};

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