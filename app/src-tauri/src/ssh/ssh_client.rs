use async_trait::async_trait;
use log::{info};
use russh::{client};
use russh::client::{DisconnectReason};
use russh::keys::key;

pub struct SshClient {
    ssh_simple_info: String,
}

impl SshClient {
    pub fn new(ssh_simple_info: String) -> Self {
        SshClient {
            ssh_simple_info
        }
    }
}

#[async_trait]
impl client::Handler for SshClient {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &key::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }

    async fn disconnected(
        &mut self,
        reason: DisconnectReason<Self::Error>,
    ) -> Result<(), Self::Error> {
        info!("{} ssh disconnected: {:?}", self.ssh_simple_info, reason);
        match reason {
            DisconnectReason::ReceivedDisconnect(_) => Ok(()),
            DisconnectReason::Error(e) => Err(e),
        }
    }
}