use std::time::Duration;

use etcd_client::{Certificate, Client, ConnectOptions, Error, GetOptions, Identity, TlsOptions};

use crate::transport::connection::Connection;
use crate::transport::kv::{SerializableKeyValue, SerializableKeyValues};

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

    pub fn get_client(&self) -> &Client {
        &self.client
    }

    pub fn has_namespace(&self) -> bool {
        if let Some(_) = self.namespace {
            true
        } else {
            false
        }
    }

    pub fn get_namespace_unchecked(&self) -> &String {
        &self.namespace.as_ref().unwrap()
    }

    pub async fn get_all_keys(&self) -> Result<SerializableKeyValues, Error> {
        let mut kv_client = self.get_client().kv_client();
        let root_path = self.get_full_key("/");
        let get_options = GetOptions::new()
            .with_prefix()
            .with_all_keys()
            .with_keys_only()
            .with_range("\0");
        let mut response = kv_client.get(root_path, Some(get_options)).await?;
        Ok(SerializableKeyValues::from_kv_vec(response.take_kvs()))
    }

    pub async fn get_key_value(&self, key: impl Into<Vec<u8>>) -> Result<SerializableKeyValue, Error> {
        let mut kv_client = self.get_client().kv_client();
        let path = self.get_full_key(key);
        let mut response = kv_client.get(path, None).await?;
        let kv = response.take_kvs();
        if kv.is_empty() {
            Err(Error::InvalidArgs(String::from("Key not found")))
        } else {
            let s_kv = SerializableKeyValue::from(kv.first().unwrap().to_owned());
            Ok(s_kv)
        }
    }

    fn get_full_key(&self, key: impl Into<Vec<u8>>) -> Vec<u8> {
        if self.has_namespace() {
            let mut full_key = self.get_namespace_unchecked().clone().into_bytes();
            full_key.append(&mut key.into());
            full_key
        } else {
            key.into()
        }
    }
}