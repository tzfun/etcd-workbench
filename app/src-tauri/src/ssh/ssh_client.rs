use std::future::Future;

use async_trait::async_trait;
use log::info;
use russh::client::{DisconnectReason, Session};
use russh::keys::ssh_key;
use russh::{client, ChannelId};

use crate::etcd::etcd_connector_handler::EtcdConnectorHandler;
use crate::transport::event::DisconnectCase;

pub struct SshClientHandler {
    ssh_user: String,
    ssh_host: String,
    ssh_port: u16,
    connector_handler: EtcdConnectorHandler,
}

impl SshClientHandler {
    pub fn new(
        ssh_user: String,
        ssh_host: String,
        ssh_port: u16,
        connector_handler: EtcdConnectorHandler,
    ) -> Self {
        SshClientHandler {
            ssh_user,
            ssh_host,
            ssh_port,
            connector_handler,
        }
    }
}

#[async_trait]
impl client::Handler for SshClientHandler {
    type Error = russh::Error;

    #[allow(unused_variables)]
    fn check_server_key(
        &mut self,
        server_public_key: &ssh_key::PublicKey,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send {
        async { Ok(true) }
    }

    /// Called when the server signals failure.
    #[allow(unused_variables)]
    fn channel_failure(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        self.connector_handler
            .disconnected(DisconnectCase::SshChannelFailure);
        async { Ok(()) }
    }

    /// Called when the server sends EOF to a channel.
    #[allow(unused_variables)]
    fn channel_eof(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        // self.connector_handler
        //     .disconnected(DisconnectCase::SshChannelEof);
        async { Ok(()) }
    }

    #[allow(unused_variables)]
    fn disconnected(
        &mut self,
        reason: DisconnectReason<Self::Error>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        async {
            info!(
                "{}@{}:{} ssh disconnected: {:?}",
                self.ssh_user, self.ssh_host, self.ssh_port, reason
            );
            match reason {
                DisconnectReason::ReceivedDisconnect(_) => {
                    Ok(())
                }
                DisconnectReason::Error(e) => {
                    self.connector_handler
                        .disconnected(DisconnectCase::SshDisconnected(e.to_string()));
                    Err(e)
                }
            }
        }
    }
}
