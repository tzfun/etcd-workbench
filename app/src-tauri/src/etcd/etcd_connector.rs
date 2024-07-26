use std::time::Duration;

use etcd_client::{Certificate, Client, ConnectOptions, Error, Identity, TlsOptions};

use crate::transport::connection::Connection;

pub struct EtcdConnector {
    namespace: Option<String>,
    client: Client,
}

impl EtcdConnector {
    pub async fn new(connection: Connection) -> Result<Self, Error> {
        let mut option = ConnectOptions::new()
            .with_keep_alive_while_idle(true)
            .with_tcp_keepalive(Duration::from_secs(5))
            .with_connect_timeout(Duration::from_secs(5))
            .with_timeout(Duration::from_secs(15));

        if let Some(user) = connection.user {
            option = option.with_user(user.username, user.password)
        };

        if let Some(tls) = connection.tls {
            let mut tls_option = TlsOptions::new();

            for cert in tls.cert {
                tls_option = tls_option.ca_certificate(Certificate::from_pem(cert.inner));
            }

            if let Some(domain) = tls.domain {
                tls_option = tls_option.domain_name(domain)
            };

            if let Some(identity) = tls.identity {
                tls_option = tls_option.identity(Identity::from_pem(identity.cert.inner, identity.key))
            };

            option = option.with_tls(tls_option)
        };
        let address = format!("{}:{}", connection.host, connection.port);
        let client = Client::connect([address], Some(option)).await?;
        Ok(EtcdConnector {
            namespace: connection.namespace,
            client,
        })
    }
}