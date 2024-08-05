use std::{fs, thread};
use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use log::{debug, warn};
use ssh2::Session;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::select;
use tokio::sync::watch;

use crate::error::LogicError;
use crate::transport::connection::ConnectionSsh;
use crate::utils::file_util;

const BUFFER_SIZE: usize = 2048;

pub struct SshTunnel {
    session: Arc<Session>,
    proxy_port: u16,
    send_abort: watch::Sender<()>,
}

impl SshTunnel {
    pub async fn new(remote: ConnectionSsh, forward_host: &'static str, forward_port: u16) -> Result<Self, LogicError> {
        let mut session = Session::new()?;
        let addr = format!("{}:{}", remote.host, remote.port);
        let tcp = TcpStream::connect(addr)?;
        session.set_tcp_stream(tcp);
        session.handshake()?;

        session.set_keepalive(false, 5);

        if let Some(identity) = remote.identity {
            if let Some(key) = identity.key {
                let file_name = file_util::create_temp_file(key.key.as_slice())?;

                debug!("Temporarily create an ssh private key file {}", file_name);

                let passphrase = if let Some(ref p) = key.passphrase {
                    Some(p.as_str())
                } else {
                    None
                };

                let res = session.userauth_pubkey_file(remote.user.as_str(), None, Path::new(&file_name), passphrase);

                fs::remove_file(file_name.clone())?;
                debug!("Deleted temp file {}", file_name);

                if let Err(e) = res {
                    return Err(LogicError::from(e));
                }
            } else if let Some(password) = identity.password {
                session.userauth_password(remote.user.as_str(), password.as_str())?;
            }
        }

        let session = Arc::new(session);
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let proxy_port = listener.local_addr()?.port();

        let (send_abort, rcv_abort) = watch::channel(());
        Self::handle_tcp_proxy(listener, Arc::clone(&session), forward_host, forward_port, rcv_abort);

        debug!("Created ssh forward accept handler");

        Ok(SshTunnel {
            session,
            proxy_port,
            send_abort,
        })
    }

    pub fn get_proxy_port(&self) -> u16 {
        self.proxy_port
    }

    fn handle_tcp_proxy(
        listener: TcpListener,
        ssh_session: Arc<Session>,
        forward_host: &'static str,
        forward_port: u16,
        rcv_abort: watch::Receiver<()>,
    ) {
        tokio::spawn(async move {
            debug!("Ssh proxy accept task started");
            let mut rcv_abort1 = rcv_abort.clone();
            let rcv_abort2 = rcv_abort.clone();

            let accept_task = async move {
                loop {
                    let accept_result = listener.accept().await;
                    match accept_result {
                        Ok((mut stream, _)) => {
                            let mut rcv_abort3 = rcv_abort2.clone();
                            let ssh_session = Arc::clone(&ssh_session);
                            debug!("Ssh proxy stream task started");
                            let session_task = async move {
                                let mut channel = ssh_session.channel_direct_tcpip(forward_host, forward_port, None).unwrap();
                                loop {
                                    let (request, size) = read_stream(&mut stream).await;
                                    if size <= 0 {
                                        break;
                                    }

                                    channel.write_all(&request[..size]).unwrap();
                                    channel.flush().unwrap();
                                    let (response, size) = read_channel(&mut channel);
                                    if size <= 0 {
                                        break;
                                    }

                                    let r = stream.write_all(&response[..size]).await;
                                    if let Err(e) = r {
                                        warn!("ssh stream write error {e}");
                                        break;
                                    }
                                    let r = stream.flush().await;
                                    if let Err(e) = r {
                                        warn!("ssh stream flush error {e}");
                                        break;
                                    }
                                }
                                let _ = channel.close();
                                debug!("Ssh proxy stream task loop finished")
                            };
                            tokio::spawn(async move {
                                select! {
                                    _stream_handle = session_task => {
                                        debug!("Ssh proxy stream task finished")
                                    }
                                    _abort = rcv_abort3.changed() => {
                                        debug!("Accept task received abort event");
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            warn!("ssh listener error: {e}");
                            break;
                        }
                    }
                };
                debug!("Ssh proxy accept loop finished");
            };
            select! {
                _accept = accept_task => {
                    debug!("Ssh proxy accept task finished")
                }
                _abort = rcv_abort1.changed() => {
                    debug!("Ssh proxy accept task received abort event");
                }
            }
        });
    }
}

impl Drop for SshTunnel {
    fn drop(&mut self) {
        match self.send_abort.send(()) {
            Ok(_) => {
                debug!("Ssh send abort success")
            }
            Err(e) => {
                warn!("Ssh send abort error: {e}")
            }
        }
        self.session.disconnect(None, "close", None)
            .unwrap_or_else(|e| warn!("Ssh session disconnect error: {e}"));
        debug!("Ssh tunnel dropped");
    }
}

async fn read_stream<R: AsyncRead + Unpin>(mut stream: R) -> (Vec<u8>, usize) {
    let mut request_buffer = vec![];
    let mut request_len = 0usize;
    loop {
        let mut buffer = vec![0; BUFFER_SIZE];

        match stream.read(&mut buffer).await {
            Ok(n) => {
                if !read_buf_bytes(&mut request_len, &mut request_buffer, n, buffer) {
                    break;
                }
            }
            Err(e) => {
                warn!("Error in reading request data: {:?}", e);
                break;
            }
        }
    }

    (request_buffer, request_len)
}

fn read_channel<R: Read>(channel: &mut R) -> (Vec<u8>, usize) {
    let mut response_buffer = vec![];
    let mut response_len = 0usize;
    loop {
        let mut buffer = vec![0; BUFFER_SIZE];
        let future_stream = channel.read(&mut buffer);
        thread::sleep(Duration::from_millis(10));

        match future_stream {
            Ok(n) => {
                if !read_buf_bytes(&mut response_len, &mut response_buffer, n, buffer) {
                    break;
                }
            }
            Err(e) => {
                if e.kind() == ErrorKind::Other {
                    debug!("Error in reading response data: {:?}", e);
                } else {
                    warn!("Error in reading response data: {:?}", e);
                }
                break;
            }
        }
    }

    (response_buffer, response_len)
}

fn read_buf_bytes(
    full_req_len: &mut usize,
    full_req_buf: &mut Vec<u8>,
    reader_buf_len: usize,
    mut reader_buf: Vec<u8>,
) -> bool {
    if reader_buf_len == 0 {
        false
    } else {
        *full_req_len += reader_buf_len;
        if reader_buf_len < BUFFER_SIZE {
            full_req_buf.append(&mut reader_buf[..reader_buf_len].to_vec());
            false
        } else {
            full_req_buf.append(&mut reader_buf);
            true
        }
    }
}