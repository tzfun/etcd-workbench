use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use etcd_client::{Certificate, Client, ConnectOptions, Error, GetOptions, Identity, PutOptions, TlsOptions};
use log::{debug};

use crate::transport::connection::Connection;
use crate::transport::kv::SerializableKeyValue;

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

    pub async fn get_all_keys(&self) -> Result<Vec<SerializableKeyValue>, Error> {
        let mut kv_client = self.get_client().kv_client();
        let root_path = self.get_full_key("/");
        let get_options = GetOptions::new()
            .with_prefix()
            .with_all_keys()
            .with_keys_only()
            .with_range("\0");
        let mut response = kv_client.get(root_path, Some(get_options)).await?;
        let kvs = response.take_kvs();
        let mut arr = Vec::with_capacity(kvs.len());
        for kv in kvs {
            arr.push(SerializableKeyValue::from(kv));
        }
        Ok(arr)
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

    pub async fn put_key_value(&self, key: impl Into<Vec<u8>>, value: impl Into<Vec<u8>>, ttl: Option<i64>) -> Result<(), Error> {
        let mut lease_id = 0;
        let final_key = self.get_full_key(key);
        if let Some(ttl_param) = ttl {
            let response = self.client.lease_client().grant(ttl_param, None).await?;
            lease_id = response.id()
        } else {
            let response = self.client.kv_client().get(final_key.clone(), Some(GetOptions::new().with_keys_only())).await?;
            let kvs = response.kvs();
            if !kvs.is_empty() {
                lease_id = kvs[0].lease();
            }
        }
        let option = if lease_id == 0 {
            None
        } else {
            Some(PutOptions::new().with_lease(lease_id))
        };

        self.client.kv_client().put(final_key, value, option).await?;

        Ok(())
    }

    pub async fn del_key(&self, keys: Vec<impl Into<Vec<u8>>>) -> Result<usize, Error> {
        let mut client = self.client.kv_client();
        let mut success = 0usize;
        for key in keys {
            let result = client.delete(self.get_full_key(key), None).await;
            if result.is_ok() {
                success += 1;
            }
        }

        Ok(success)
    }

    pub async fn get_kv_history_versions(&self, key: impl Into<Vec<u8>>, start: i64, end: i64) -> Result<Vec<i64>, Error> {
        let mut history = Vec::new();
        let final_key = self.get_full_key(key);
        self.get_kv_history_versions0(final_key, end, start, end, &mut history).await?;
        Ok(history)
    }

    fn get_kv_history_versions0<'a>(
        &'a self,
        key: Vec<u8>,
        revision: i64,
        start: i64,
        end: i64,
        history: &'a mut Vec<i64>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>> {
        Box::pin(async move {
            if revision >= start && revision <= end {
                let result = self.client.kv_client().get(key.clone(), Some(GetOptions::new().with_keys_only().with_revision(revision))).await;
                match result {
                    Ok(response) => {
                        let kvs = response.kvs();
                        if kvs.is_empty() {
                            return Ok(());
                        }
                        let kv = &kvs[0];
                        let next_revision = if revision >= kv.create_revision() && revision <= kv.mod_revision() {
                            history.push(revision);
                            revision - 1
                        } else {
                            kv.mod_revision()
                        };
                        self.get_kv_history_versions0(key, next_revision, start, end, history).await?;
                        Ok(())
                    }
                    Err(e) => {
                        debug!("get revision error: {e}");
                        return Ok(());
                    }
                }
            } else {
                Ok(())
            }
        })
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