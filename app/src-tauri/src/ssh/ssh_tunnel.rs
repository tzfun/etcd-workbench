use std::{fs, thread};
use std::env::temp_dir;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use log::{debug, warn};
use uuid::Uuid;

use crate::error::LogicError;
use crate::transport::connection::ConnectionSsh;

const BUFFER_SIZE: usize = 2048;

pub struct SshTunnel {
    session: Arc<ssh2::Session>,
    proxy_listener: Arc<TcpListener>,
    proxy_port: u16,
}

impl SshTunnel {
    pub fn new(remote: ConnectionSsh, forward_host: &'static str, forward_port: u16) -> Result<Self, LogicError> {
        let mut session = ssh2::Session::new()?;
        let addr = format!("{}:{}", remote.host, remote.port);
        let tcp = TcpStream::connect(addr)?;
        session.set_tcp_stream(tcp);
        session.handshake()?;

        session.set_keepalive(false, 5);

        if let Some(identity) = remote.identity {
            if let Some(key) = identity.key {
                let mut dir = temp_dir();
                let private_key_file = format!("{}", Uuid::new_v4());
                dir.push(private_key_file);
                let file_name = dir.display().to_string();
                let mut file = File::create(dir)?;
                file.write(key.key.as_slice())?;

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

        let listener = TcpListener::bind("127.0.0.1:0")?;
        let proxy_port = listener.local_addr()?.port();
        let ssh_session = Arc::clone(&session);
        let listener = Arc::new(listener);

        let proxy_listener = Arc::clone(&listener);
        thread::Builder::new()
            .name(format!("ssh-accept-{}", forward_port))
            .spawn(move || {
                for stream in listener.incoming() {
                    let mut stream = stream.unwrap();

                    let ssh_session = Arc::clone(&ssh_session);
                    thread::spawn(move || {
                        let mut channel = ssh_session.channel_direct_tcpip(forward_host, forward_port, None).unwrap();
                        loop {
                            let (request, size) = read_stream(&mut stream);
                            if size <= 0 {
                                break;
                            }

                            channel.write_all(&request[..size]).unwrap();
                            channel.flush().unwrap();

                            let (response, size) = read_channel(&mut channel);
                            if size <= 0 {
                                break;
                            }

                            stream.write_all(&response[..size]).unwrap();
                            stream.flush().unwrap();
                        }
                        channel.close().unwrap();
                    });

                }
                debug!("Accept thread closed");
            })?;
        debug!("Created ssh forward accept thread");

        Ok(SshTunnel {
            session,
            proxy_port,
            proxy_listener
        })
    }

    pub fn get_proxy_port(&self) -> u16 {
        self.proxy_port
    }

    pub fn close(self) {
        let listener = self.proxy_listener;
        drop(listener);
    }
}

fn read_stream<R: Read>(mut stream: R) -> (Vec<u8>, usize) {
    let mut request_buffer = vec![];
    let mut request_len = 0usize;
    loop {
        let mut buffer = vec![0; BUFFER_SIZE];

        match stream.read(&mut buffer) {
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
                warn!("Error in reading response data: {:?}", e);
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