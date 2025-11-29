use std::borrow::Cow;
use std::io::{Error, ErrorKind};
use std::sync::Arc;
use std::time::Duration;

use crate::api::settings::get_settings;
use crate::error::LogicError;
use crate::etcd::etcd_connector_handler::EtcdConnectorHandler;
use crate::ssh::ssh_client::SshClientHandler;
use crate::transport::connection::ConnectionSsh;
use crate::transport::event::DisconnectCase;
use log::{debug, error, info, warn};
use russh::client::{AuthResult, Handle, Msg};
use russh::keys::key::PrivateKeyWithHashAlg;
use russh::keys::{decode_secret_key, HashAlg};
use russh::{client, kex, Channel, ChannelMsg, Preferred};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{oneshot, watch};
use tokio::time::timeout;
use tokio::{io, select};

pub struct SshTunnel {
    proxy_port: u16,
    send_abort: watch::Sender<()>,
}

impl SshTunnel {
    pub async fn new(
        ssh_config: ConnectionSsh,
        forward_host: &'static str,
        forward_port: u16,
        handler: EtcdConnectorHandler,
    ) -> Result<Self, LogicError> {
        let config = client::Config {
            inactivity_timeout: None,
            keepalive_interval: Some(Duration::from_secs(10)),
            keepalive_max: 3,
            preferred: Preferred {
                kex: Cow::Borrowed(&[
                    kex::CURVE25519,
                    kex::CURVE25519_PRE_RFC_8731,
                    kex::DH_G1_SHA1,
                    kex::DH_G14_SHA1,
                    kex::DH_G16_SHA512,
                    kex::DH_G14_SHA256,
                    kex::ECDH_SHA2_NISTP256,
                    kex::ECDH_SHA2_NISTP384,
                    kex::ECDH_SHA2_NISTP521,
                    kex::EXTENSION_SUPPORT_AS_CLIENT,
                    kex::EXTENSION_SUPPORT_AS_SERVER,
                    kex::EXTENSION_OPENSSH_STRICT_KEX_AS_CLIENT,
                    kex::EXTENSION_OPENSSH_STRICT_KEX_AS_SERVER,
                ]),
                ..<_>::default()
            },
            ..<_>::default()
        };

        let config = Arc::new(config);

        let ssh_handler = SshClientHandler::new(
            ssh_config.user.clone(),
            ssh_config.host.clone(),
            ssh_config.port,
            handler.clone(),
        );

        let ssh_simple_info = format!(
            "{}@{}:{}",
            ssh_config.user, ssh_config.host, ssh_config.port
        );
        let addr = format!("{}:{}", ssh_config.host, ssh_config.port);

        let settings = get_settings().await?;

        let stream = timeout(
            Duration::from_secs(settings.ssh_connect_timeout_seconds),
            TcpStream::connect(addr),
        )
        .await
        .map_err(|_| io::Error::new(ErrorKind::ConnectionAborted, "ssh connection timeout"))??;

        let mut session = client::connect_stream(config, stream, ssh_handler).await?;

        if let Some(identity) = ssh_config.identity {
            if let Some(key) = identity.key {
                let passphrase = if let Some(ref p) = key.passphrase {
                    Some(p.as_str())
                } else {
                    None
                };
                let hash_alg = key
                    .hash_algorithm
                    .clone()
                    .map(|s| HashAlg::new(s.as_str()).unwrap());

                match decode_secret_key(String::from_utf8(key.key)?.as_str(), passphrase) {
                    Ok(key_pair) => {
                        let private_key = PrivateKeyWithHashAlg::new(Arc::new(key_pair), hash_alg);
                        let res = session
                            .authenticate_publickey(ssh_config.user, private_key)
                            .await?;
                        Self::handle_auth_result(res)?;
                    }
                    Err(e) => {
                        error!("decode ssh key failed: {}", e);
                        return Err(LogicError::IoError(Error::new(
                            ErrorKind::ConnectionAborted,
                            "Failed to parse ssh private key",
                        )));
                    }
                }
            } else if let Some(password) = identity.password {
                let res = session
                    .authenticate_password(ssh_config.user, password)
                    .await?;
                Self::handle_auth_result(res)?;
            }
        }

        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let proxy_port = listener.local_addr()?.port();

        let (send_abort, rcv_abort) = watch::channel(());

        info!(
            "{} create ssh forward accept handler, local port is {}",
            ssh_simple_info, proxy_port
        );

        Self::handle_tcp_proxy(
            ssh_simple_info,
            listener,
            Arc::new(session),
            forward_host,
            forward_port,
            rcv_abort,
            handler,
        )
        .await?;

        Ok(SshTunnel {
            proxy_port,
            send_abort,
        })
    }

    fn handle_auth_result(res: AuthResult) -> Result<(), LogicError> {
        match res {
            client::AuthResult::Failure {
                remaining_methods,
                partial_success,
            } => {
                debug!(
                    "Ssh authentication failed, methods:{:?}, partial_success: {}",
                    remaining_methods, partial_success
                );
                Err(LogicError::IoError(Error::new(
                    ErrorKind::ConnectionAborted,
                    "SSH connection rejected: authentication failure",
                )))
            }
            _ => Ok(()),
        }
    }

    pub fn get_proxy_port(&self) -> u16 {
        self.proxy_port
    }

    async fn handle_tcp_proxy(
        ssh_simple_info: String,
        listener: TcpListener,
        ssh_session: Arc<Handle<SshClientHandler>>,
        forward_host: &'static str,
        forward_port: u16,
        rcv_abort: watch::Receiver<()>,
        handler: EtcdConnectorHandler,
    ) -> Result<(), LogicError> {
        let (sender, receiver) = oneshot::channel();
        tokio::spawn(async move {
            let mut rcv_abort1 = rcv_abort.clone();
            let rcv_abort2 = rcv_abort.clone();

            let ssh_simple_info1 = Arc::new(ssh_simple_info);
            let ssh_simple_info2 = Arc::clone(&ssh_simple_info1);

            let accept_task = async move {
                {
                    sender.send(()).unwrap();
                }
                debug!("{} ssh accept future start", ssh_simple_info2);
                let local_port = listener.local_addr().unwrap().port();
                loop {
                    let accept_result = listener.accept().await;
                    match accept_result {
                        Ok((stream, addr)) => {
                            let rcv_abort3 = rcv_abort2.clone();
                            let ssh_session = Arc::clone(&ssh_session);
                            let ssh_simple_info3 = Arc::clone(&ssh_simple_info2);

                            debug!("ssh proxy stream task started, chain: local({}) -> local(127.0.0.1:{}) -> ssh({}) -> remote({}:{})",
                                addr, local_port, ssh_simple_info2,  forward_host, forward_port);

                            let direct_channel_result = ssh_session
                                .channel_open_direct_tcpip(
                                    forward_host,
                                    forward_port as u32,
                                    "127.0.0.1",
                                    57128,
                                )
                                .await;

                            match direct_channel_result {
                                Ok(channel) => {
                                    tokio::spawn(async move {
                                        let _ = Self::start_ssh_tunnel(
                                            channel,
                                            stream,
                                            rcv_abort3,
                                            ssh_simple_info3,
                                        )
                                        .await;
                                    });
                                }
                                Err(e) => {
                                    error!("Unable to forward messages via ssh: {e}");
                                    handler.disconnected(DisconnectCase::SshTunnelError(
                                        e.to_string(),
                                    ));
                                    continue;
                                }
                            }
                        }
                        Err(e) => {
                            warn!("ssh proxy listener error: {e}");
                            handler.disconnected(DisconnectCase::SshTunnelError(e.to_string()));
                            break;
                        }
                    }
                }
                debug!("ssh proxy accept loop finished | {}", ssh_simple_info2);
            };

            select! {
                _accept = accept_task => {
                    debug!("{} ssh proxy accept task finished", ssh_simple_info1)
                }
                _abort = rcv_abort1.changed() => {
                    debug!("ssh proxy accept task received abort event | {}", ssh_simple_info1);
                }
            }
            debug!("ssh accept future finished | {}", ssh_simple_info1);
        });

        let _ = receiver.await?;
        Ok(())
    }

    pub async fn start_ssh_tunnel(
        channel: Channel<Msg>, // russh channel
        mut stream: TcpStream,     // TCP 客户端
        mut rcv_abort: watch::Receiver<()>,
        ssh_simple_info: Arc<String>,
    ) -> Result<(), LogicError> {
        let (mut socket_reader, mut socket_writer) = stream.split();

        let (mut channel_reader, channel_writer) = channel.split();

        // 缓冲区
        let mut buf_socket_to_ssh = vec![0u8; 8192];

        // --- TCP → SSH ---
        let to_ssh = async {
            loop {
                match socket_reader.read(&mut buf_socket_to_ssh).await {
                    Ok(0) => {
                        // TCP 已断开
                        log::info!(
                            "tcp disconnected, sending SSH channel EOF | {}",
                            ssh_simple_info
                        );

                        let _ = channel_writer.eof().await;
                        let _ = channel_writer.close().await;
                        break;
                    }
                    Ok(n) => {
                        if n > 0 {
                            if let Err(e) = channel_writer.data(&buf_socket_to_ssh[..n]).await {
                                log::error!(
                                    "ssh channel write error: {:?} | {}",
                                    e,
                                    ssh_simple_info
                                );
                                break;
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => {
                        log::error!("socket -> ssh read error: {:?} | {}", e, ssh_simple_info);
                        break;
                    }
                }
            }
        };

        // --- SSH → TCP ---
        let to_tcp = async {
            loop {
                match channel_reader.wait().await {
                    Some(ChannelMsg::Eof) | Some(ChannelMsg::Close) | None => {
                        // SSH channel EOF
                        log::info!("ssh channel EOF, closing TCP side | {}", ssh_simple_info);
                        let _ = socket_writer.shutdown().await;
                        break;
                    }
                    Some(ChannelMsg::Data { data }) => {
                        if let Err(e) = socket_writer.write_all(&data).await {
                            log::error!("tcp write error: {:?} | {}", e, ssh_simple_info);
                            break;
                        }
                    }
                    _ => {}
                }
            }
        };

        // 并发执行两端
        tokio::select! {
            _ = to_ssh => {
                debug!("TCP → SSH stream proxy stopped | {}", ssh_simple_info)
            },
            _ = to_tcp => {
                debug!("SSH → TCP stream proxy stopped | {}", ssh_simple_info)
            },
            _ = rcv_abort.changed() => {
                debug!("ssh proxy stream task received abort event | {}", ssh_simple_info)
            }
        }

        log::info!("ssh tunnel finished. | {}", ssh_simple_info);

        Ok(())
    }
}

impl Drop for SshTunnel {
    fn drop(&mut self) {
        match self.send_abort.send(()) {
            Ok(_) => {
                debug!("ssh send abort success")
            }
            Err(e) => {
                warn!("ssh send abort error: {e}")
            }
        }
        debug!("drop ssh tunnel");
    }
}
