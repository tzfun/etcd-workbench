use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use etcd_client::{Certificate, Client, ConnectOptions, Error, GetOptions, Identity, Permission, PermissionType, PutOptions, RoleRevokePermissionOptions, TlsOptions};
use log::{debug, warn};

use crate::transport::connection::Connection;
use crate::transport::kv::SerializableKeyValue;
use crate::transport::user::{SerializablePermission, SerializableUser};

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

    pub async fn kv_get_all_keys(&self) -> Result<Vec<SerializableKeyValue>, Error> {
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

    pub async fn kv_get(&self, key: impl Into<Vec<u8>>) -> Result<SerializableKeyValue, Error> {
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

    pub async fn kv_put(&self, key: impl Into<Vec<u8>>, value: impl Into<Vec<u8>>, ttl: Option<i64>) -> Result<(), Error> {
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

    pub async fn kv_delele(&self, keys: Vec<impl Into<Vec<u8>>>) -> Result<usize, Error> {
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

    pub async fn kv_get_history_versions(&self, key: impl Into<Vec<u8>>, start: i64, end: i64) -> Result<Vec<i64>, Error> {
        let mut history = Vec::new();
        let final_key = self.get_full_key(key);
        self.kv_get_history_versions0(final_key, end, start, end, &mut history).await;
        Ok(history)
    }

    fn kv_get_history_versions0<'a>(
        &'a self,
        key: Vec<u8>,
        revision: i64,
        start: i64,
        end: i64,
        history: &'a mut Vec<i64>,
    ) -> Pin<Box<dyn Future<Output=()> + 'a>> {
        Box::pin(async move {
            if revision >= start && revision <= end {
                let result = self.client.kv_client().get(key.clone(), Some(GetOptions::new().with_keys_only().with_revision(revision))).await;
                match result {
                    Ok(response) => {
                        let kvs = response.kvs();
                        if kvs.is_empty() {
                            return ();
                        }
                        let kv = &kvs[0];
                        let next_revision = if revision >= kv.create_revision() && revision <= kv.mod_revision() {
                            history.push(revision);
                            revision - 1
                        } else {
                            kv.mod_revision()
                        };
                        self.kv_get_history_versions0(key, next_revision, start, end, history).await;
                        ()
                    }
                    Err(e) => {
                        debug!("get revision error: {e}");
                        return ();
                    }
                }
            } else {
                ()
            }
        })
    }

    pub async fn user_list(&self) -> Result<Vec<SerializableUser>, Error> {
        let mut auth_client = self.client.auth_client();
        let response = auth_client.user_list().await?;
        let users = response.users();
        let mut result_users = Vec::with_capacity(users.len());
        for user in response.users() {
            let response = auth_client.user_get(user).await?;
            result_users.push(SerializableUser {
                user: user.clone(),
                roles: Vec::from(response.roles()),
            })
        }
        Ok(result_users)
    }

    pub async fn user_add(&self, user: String, password: String) -> Result<(), Error> {
        self.client.auth_client().user_add(user, password, None).await?;
        Ok(())
    }

    pub async fn user_delete(&self, user: String) -> Result<(), Error> {
        self.client.auth_client().user_delete(user).await?;
        Ok(())
    }

    pub async fn user_change_password(&self, user: String, password: String) -> Result<(), Error> {
        self.client.auth_client().user_change_password(user, password).await?;
        Ok(())
    }

    pub async fn user_grant_role(&self, user: String, role: String) -> Result<(), Error> {
        self.client.auth_client().user_grant_role(user, role).await?;
        Ok(())
    }

    pub async fn user_revoke_role(&self, user: String, role: String) -> Result<(), Error> {
        self.client.auth_client().user_revoke_role(user, role).await?;
        Ok(())
    }

    pub async fn user_is_root(&self, user: String) -> Result<bool, Error> {
        if user == "root" {
            return Ok(true);
        }

        let response = self.client.auth_client().user_get(user).await?;
        for role in response.roles() {
            if role == "root" {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub async fn auth_enable(&self) -> Result<(), Error> {
        self.client.auth_client().auth_enable().await?;
        Ok(())
    }

    pub async fn auth_disable(&self) -> Result<(), Error> {
        self.client.auth_client().auth_disable().await?;
        Ok(())
    }

    pub async fn role_list(&self) -> Result<Vec<String>, Error> {
        let response = self.client.auth_client().role_list().await?;
        Ok(Vec::from(response.roles()))
    }

    pub async fn role_get_permissions(&self, role: String) -> Result<Vec<SerializablePermission>, Error> {
        let response = self.client.auth_client().role_get(role).await?;
        let permissions = response.permissions();
        let mut result = Vec::with_capacity(permissions.len());

        for permission in permissions {
            let key_bytes = permission.key();
            let key = String::from(key_bytes);
            let perm_type = PermissionType::try_from(permission.get_type()).unwrap_or_else(|p| {
                warn!("Catch a unknown enum value in PermissionType: {}", p);
                PermissionType::Read
            });
            let range_end = permission.range_end();

            let prefix = permission.is_prefix();

            let key_bytes_len = key_bytes.len();
            let range_end_len = range_end.len();
            //  为兼容老版本的etcd，空字符串是一个长度为1且内容为0的byte数组
            let all_keys = (key_bytes_len == 0 && range_end_len == 0)
                || (key_bytes_len == 1 && range_end_len == 1 && key_bytes[0] == 0 && range_end[0] == 0);

            result.push(SerializablePermission {
                key,
                perm_type,
                prefix,
                all_keys,
            })
        }

        Ok(result)
    }

    pub async fn role_add(&self, role: String) -> Result<(), Error> {
        self.client.auth_client().role_add(role).await?;
        Ok(())
    }

    pub async fn role_delete(&self, role: String) -> Result<(), Error> {
        self.client.auth_client().role_delete(role).await?;
        Ok(())
    }

    pub async fn role_grant_permission(
        &self,
        role: String,
        permission: SerializablePermission,
    ) -> Result<(), Error> {
        self.client.auth_client().role_grant_permission(role, permission.into()).await?;
        Ok(())
    }

    pub async fn role_revoke_permission(&self, role: String, permission: SerializablePermission,) -> Result<(), Error> {
        let range_ned = permission.parse_range_end();
        self.client.auth_client()
            .role_revoke_permission(role, permission.key, Some(RoleRevokePermissionOptions::new().with_range_end(range_ned)))
            .await?;

        Ok(())
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