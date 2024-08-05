#![allow(unused)]
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use etcd_client::{AlarmAction, AlarmType, Certificate, Client, ConnectOptions, Error, GetOptions, Identity, PutOptions, RoleRevokePermissionOptions, TlsOptions};
use log::debug;
use crate::error::LogicError;
use crate::ssh::ssh_tunnel::SshTunnel;
use crate::transport::connection::Connection;
use crate::transport::kv::SerializableKeyValue;
use crate::transport::maintenance::{SerializableCluster, SerializableClusterMember, SerializableClusterStatus};
use crate::transport::user::{SerializablePermission, SerializableUser};

pub struct EtcdConnector {
    namespace: Option<String>,
    client: Client,
    ssh: Option<SshTunnel>
}

impl EtcdConnector {
    pub async fn new(connection: Connection) -> Result<Self, LogicError> {
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
                tls_option = tls_option.ca_certificate(Certificate::from_pem(cert));
            }

            if let Some(domain) = tls.domain {
                tls_option = tls_option.domain_name(domain)
            };

            if let Some(identity) = tls.identity {
                tls_option = tls_option.identity(Identity::from_pem(identity.cert, identity.key))
            };

            option = option.with_tls(tls_option)
        };
        let host = Box::leak(connection.host.into_boxed_str());
        let mut port = connection.port;
        let namespace = connection.namespace.clone();

        let ssh = if let Some(ssh) = connection.ssh {
            let ssh_context = SshTunnel::new(ssh, host, port).await?;
            port = ssh_context.get_proxy_port();
            Some(ssh_context)
        } else {
            None
        };

        let address = format!("{}:{}", host, port);
        let client = Client::connect([address], Some(option)).await?;
        Ok(EtcdConnector {
            namespace,
            client,
            ssh
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

    pub async fn test_connection(&self) -> Result<(), Error> {
        let key = self.get_full_key("/");
        self.client.kv_client().get(key, Some(GetOptions::new().with_keys_only())).await?;
        Ok(())
    }

    /// 获取所有key，不包含value
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

    /// 获取键值对详情
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

    /// 获取Key的数量
    pub async fn kv_count(&self) -> Result<i64, Error> {
        let key = self.get_full_key("/");
        let response = self.client.kv_client().get(key, Some(GetOptions::new().with_count_only())).await?;
        Ok(response.count())
    }

    /// 更新键值对
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

    /// 删除键值对
    pub async fn kv_delete(&self, keys: Vec<impl Into<Vec<u8>>>) -> Result<usize, Error> {
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

    /// 获取某一个key的历史版本，如果中间某个版本以及被删除或压缩，将终止搜索
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

    fn get_full_key(&self, key: impl Into<Vec<u8>>) -> Vec<u8> {
        if self.has_namespace() {
            let mut full_key = self.get_namespace_unchecked().clone().into_bytes();
            full_key.append(&mut key.into());
            full_key
        } else {
            key.into()
        }
    }

    /// 查询所有用户
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

    /// 新增用户并设置密码
    pub async fn user_add(&self, user: String, password: String) -> Result<(), Error> {
        self.client.auth_client().user_add(user, password, None).await?;
        Ok(())
    }

    /// 删除用户
    pub async fn user_delete(&self, user: String) -> Result<(), Error> {
        self.client.auth_client().user_delete(user).await?;
        Ok(())
    }

    /// 修改用户密码
    pub async fn user_change_password(&self, user: String, password: String) -> Result<(), Error> {
        self.client.auth_client().user_change_password(user, password).await?;
        Ok(())
    }

    /// 给用户授权角色
    pub async fn user_grant_role(&self, user: String, role: String) -> Result<(), Error> {
        self.client.auth_client().user_grant_role(user, role).await?;
        Ok(())
    }

    /// 回收用户的角色
    pub async fn user_revoke_role(&self, user: String, role: String) -> Result<(), Error> {
        self.client.auth_client().user_revoke_role(user, role).await?;
        Ok(())
    }

    /// 判断用户是否是 root 用户（拥有root角色权限的用户也被认为是root用户）
    pub async fn user_is_root(&self, user: &String) -> Result<bool, Error> {
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

    /// 开启权限验证功能，此功能调用后可能会导致connector无法使用
    pub async fn auth_enable(&self) -> Result<(), Error> {
        self.client.auth_client().auth_enable().await?;
        Ok(())
    }

    /// 关闭权限验证功能，此功能调用后可能会导致connector无法使用
    pub async fn auth_disable(&self) -> Result<(), Error> {
        self.client.auth_client().auth_disable().await?;
        Ok(())
    }

    /// 获取所有角色
    pub async fn role_list(&self) -> Result<Vec<String>, Error> {
        let response = self.client.auth_client().role_list().await?;
        Ok(Vec::from(response.roles()))
    }

    /// 获取角色的权限信息
    pub async fn role_get_permissions(&self, role: String) -> Result<Vec<SerializablePermission>, Error> {
        let response = self.client.auth_client().role_get(role).await?;
        let permissions = response.permissions();
        let mut result = Vec::with_capacity(permissions.len());

        for permission in permissions {
            let key_bytes = permission.key();
            let key = String::from_utf8(Vec::from(key_bytes)).unwrap();
            let perm_type = permission.get_type();
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

    /// 添加新角色
    pub async fn role_add(&self, role: String) -> Result<(), Error> {
        self.client.auth_client().role_add(role).await?;
        Ok(())
    }

    /// 删除角色
    pub async fn role_delete(&self, role: String) -> Result<(), Error> {
        self.client.auth_client().role_delete(role).await?;
        Ok(())
    }

    /// 给角色授权权限
    pub async fn role_grant_permission(
        &self,
        role: String,
        permission: SerializablePermission,
    ) -> Result<(), Error> {
        self.client.auth_client().role_grant_permission(role, permission.into()).await?;
        Ok(())
    }

    /// 回收角色的权限
    pub async fn role_revoke_permission(&self, role: String, permission: SerializablePermission) -> Result<(), Error> {
        let range_ned = permission.parse_range_end();
        self.client.auth_client()
            .role_revoke_permission(role, permission.key, Some(RoleRevokePermissionOptions::new().with_range_end(range_ned)))
            .await?;

        Ok(())
    }

    /// 获取集群的详情信息，包含集群数据、成员、报警、状态等信息
    pub async fn cluster_get(&self) -> Result<SerializableCluster, Error> {
        let mut response = self.client.cluster_client().member_list().await?;
        let status = self.client.maintenance_client().status().await?;

        let cluster_status = SerializableClusterStatus {
            version: String::from(status.version()),
            db_size: status.db_size(),
            raft_used_db_size: status.raft_used_db_size(),
            leader: status.leader().to_string(),
            raft_index: status.raft_index().to_string(),
            raft_term: status.raft_term().to_string(),
            raft_applied_index: status.raft_applied_index().to_string(),
            errors: Vec::from(status.errors()),
        };
        let alarm_response = self.client.maintenance_client().alarm(AlarmAction::Get, AlarmType::None, None).await?;
        let alarms = alarm_response.alarms();

        let mut alarms_map = HashMap::with_capacity(alarms.len());
        for alarm in alarms {
            alarms_map.insert(alarm.member_id(), alarm.alarm());
        }

        let pb_members = response.members();
        let mut members = Vec::with_capacity(pb_members.len());
        for member in pb_members {
            let id = member.id().to_string();
            let name = String::from(member.name());

            members.push(SerializableClusterMember {
                id,
                name,
                peer_uri: member.peer_urls().to_vec(),
                client_uri: member.client_urls().to_vec(),
                alarm_type: *alarms_map.get(&member.id()).unwrap_or_else(|| &AlarmType::None) as i32,
            })
        }

        let header = response.take_header().unwrap();

        Ok(SerializableCluster {
            id: header.cluster_id().to_string(),
            revision: header.revision(),
            raft_term: header.raft_term().to_string(),
            members,
            status: cluster_status,
        })
    }

    /// 集群添加新成员节点
    pub async fn cluster_add_member(&self, urls: impl Into<Vec<String>>) -> Result<(), Error> {
        self.client.cluster_client().member_add(urls, None).await?;
        Ok(())
    }

    /// 集群移除成员节点
    pub async fn cluster_remove_member(&self, id: String) -> Result<(), Error> {
        match id.parse::<u64>() {
            Ok(id) => {
                self.client.cluster_client().member_remove(id).await?;
                Ok(())
            }
            Err(e) => {
                Err(Error::InvalidArgs(e.to_string()))
            }
        }
    }

    /// 集群更新成员节点
    pub async fn cluster_update_member(&self, id: String, urls: impl Into<Vec<String>>) -> Result<(), Error> {
        match id.parse::<u64>() {
            Ok(id) => {
                self.client.cluster_client().member_update(id, urls).await?;
                Ok(())
            }
            Err(e) => {
                Err(Error::InvalidArgs(e.to_string()))
            }
        }
    }

    /// 对节点进行碎片整理。这是一个比较消耗资源的操作，谨慎调用。
    pub async fn maintenance_defragment(&self) -> Result<(), Error> {
        self.client.maintenance_client().defragment().await?;
        Ok(())
    }
}