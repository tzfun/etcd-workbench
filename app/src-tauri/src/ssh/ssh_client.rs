use std::future::Future;

use async_trait::async_trait;
use log::info;
use russh::client;
use russh::client::DisconnectReason;
use russh::keys::ssh_key;

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

    #[allow(unused_variables)]
    fn check_server_key(
        &mut self,
        server_public_key: &ssh_key::PublicKey,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send {
        async { Ok(true) }
    }

    #[allow(unused_variables)]
    fn disconnected(
        &mut self,
        reason: DisconnectReason<Self::Error>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        async {
            info!("{} ssh disconnected: {:?}", self.ssh_simple_info, reason);
            match reason {
                DisconnectReason::ReceivedDisconnect(_) => Ok(()),
                DisconnectReason::Error(e) => Err(e),
            }
        }
    }
}