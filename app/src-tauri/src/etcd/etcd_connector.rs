#![allow(unused)]
use std::any::Any;
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::time::Duration;
use std::{fs, u8};

use crate::api::settings::get_settings;
use crate::error::LogicError;
use crate::etcd::wrapped_etcd_client::WrappedEtcdClient;
use crate::ssh::ssh_tunnel::SshTunnel;
use crate::transport::connection::{Connection, ConnectionUser};
use crate::transport::kv::{
    SerializableKeyValue, SerializableLeaseInfo, SerializableLeaseSimpleInfo,
};
use crate::transport::maintenance::{
    SerializableCluster, SerializableClusterMember, SerializableClusterStatus, SnapshotInfo,
    SnapshotState,
};
use crate::transport::user::{SerializablePermission, SerializableUser};
use crate::utils::k8s_formatter;
use etcd_client::{
    AlarmAction, AlarmType, Certificate, Client, ConnectOptions, Error, GetOptions, GetResponse,
    Identity, LeaseGrantOptions, LeaseTimeToLiveOptions, PutOptions, RoleRevokePermissionOptions,
    SortOrder, SortTarget, TlsOptions,
};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::select;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

pub struct EtcdConnector {
    namespace: Option<String>,
    client: WrappedEtcdClient,
    ssh: Option<SshTunnel>,
}

impl EtcdConnector {
    pub async fn new(connection: Connection) -> Result<Self, LogicError> {
        let settings = get_settings().await?;

        let mut option = ConnectOptions::new()
            .with_keep_alive_while_idle(true)
            .with_tcp_keepalive(Duration::from_secs(5))
            .with_connect_timeout(Duration::from_secs(settings.connect_timeout_seconds))
            .with_timeout(Duration::from_secs(settings.request_timeout_seconds));

        if let Some(user) = connection.user.clone() {
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
        let mut host = connection.host;
        let mut port = connection.port;
        let namespace = connection.namespace.clone();

        let ssh = if let Some(ssh) = connection.ssh {
            let ssh_context =
                SshTunnel::new(ssh, Box::leak(host.clone().into_boxed_str()), port).await?;
            port = ssh_context.get_proxy_port();
            host.clear();
            host.push_str("127.0.0.1");
            Some(ssh_context)
        } else {
            None
        };

        let address = format!("{}:{}", host, port);
        info!("Connect to etcd server: {}", address);
        let client = Client::connect([address], Some(option)).await?;
        Ok(EtcdConnector {
            namespace,
            client: WrappedEtcdClient::new(client, connection.user),
            ssh,
        })
    }

    pub fn has_namespace(&self) -> bool {
        if let Some(ref namespace) = self.namespace {
            !namespace.is_empty()
        } else {
            false
        }
    }

    pub fn get_namespace_unchecked(&self) -> &String {
        &self.namespace.as_ref().unwrap()
    }

    pub async fn test_connection(&self) -> Result<(), Error> {
        let key = self.prefix_namespace("/");
        let response = self
            .client
            .get_inner()
            .kv_client()
            .get(key, Some(GetOptions::new().with_keys_only()))
            .await?;
        debug!("test connection, kv length: {}", response.kvs().len());
        Ok(())
    }

    /// 获取所有key，不包含value
    pub async fn kv_get_all_keys(&mut self) -> Result<Vec<SerializableKeyValue>, Error> {
        let root_path = self.root_key();
        let get_options = GetOptions::new().with_prefix().with_keys_only();
        self.kv_get_by_option(root_path, Some(get_options)).await
    }

    /// 分页获取所有key，不包含value
    pub async fn kv_get_all_keys_paging(
        &mut self,
        cursor_key: impl Into<Vec<u8>>,
        limit: i64,
    ) -> Result<Vec<SerializableKeyValue>, Error> {
        let mut cursor: Vec<u8> = cursor_key.into();
        cursor.push(0);

        let key = self.prefix_namespace(cursor);
        let end_key = self.prefix_namespace_to_range_end(vec![0]);

        let get_options = GetOptions::new()
            .with_keys_only()
            .with_range(end_key)
            .with_limit(limit)
            .with_sort(SortTarget::Key, SortOrder::Ascend);
        self.kv_get_by_option(key, Some(get_options)).await
    }

    async fn kv_get_by_option(
        &mut self,
        key: Vec<u8>,
        option: Option<GetOptions>,
    ) -> Result<Vec<SerializableKeyValue>, Error> {
        let mut response = self.client.kv_get_request(key, option).await?;

        let kvs = response.take_kvs();
        let mut arr = Vec::with_capacity(kvs.len());
        for kv in kvs {
            let mut s_kv = SerializableKeyValue::from(kv);
            if let Some(namespace) = &self.namespace {
                s_kv.remove_prefix(namespace);
            }
            arr.push(s_kv);
        }
        Ok(arr)
    }

    /// 请求Key-Value
    pub async fn kv_get_request(
        &mut self,
        key: impl Into<Vec<u8>>,
        option: Option<GetOptions>,
    ) -> Result<GetResponse, Error> {
        let path = self.prefix_namespace(key);
        self.client.kv_get_request(path, option).await
    }

    /// 获取键值对详情
    pub async fn kv_get(
        &mut self,
        key: impl Into<Vec<u8>>,
    ) -> Result<SerializableKeyValue, LogicError> {
        let path = self.prefix_namespace(key);
        let kv = self.kv_get_by_option(path, None).await?;

        self.find_first_kv(kv)
    }

    /// 根据历史版本获取键值对详情
    pub async fn kv_get_by_version(
        &mut self,
        key: impl Into<Vec<u8>>,
        version: i64,
    ) -> Result<SerializableKeyValue, LogicError> {
        let path = self.prefix_namespace(key);
        let kv = self
            .kv_get_by_option(path, Some(GetOptions::new().with_revision(version)))
            .await?;

        self.find_first_kv(kv)
    }

    fn find_first_kv(
        &self,
        kv: Vec<SerializableKeyValue>,
    ) -> Result<SerializableKeyValue, LogicError> {
        if kv.is_empty() {
            Err(LogicError::ResourceNotExist(
                "The key does not exist or has expired.",
            ))
        } else {
            let mut s_kv = kv[0].clone();
            let full_key = s_kv.key.clone();
            if let Some(namespace) = &self.namespace {
                s_kv.remove_prefix(namespace);
            }

            s_kv.formatted_value = k8s_formatter::try_format_value(&full_key, &s_kv.value);
            Ok(s_kv)
        }
    }

    /// 获取Key的数量
    pub async fn kv_count(&mut self) -> Result<i64, Error> {
        let key = self.prefix_namespace("/");
        let response = self
            .client
            .kv_get_request(key, Some(GetOptions::new().with_count_only()))
            .await?;
        Ok(response.count())
    }

    /// 更新键值对
    pub async fn kv_put(
        &mut self,
        key: impl Into<Vec<u8>>,
        value: impl Into<Vec<u8>>,
        ttl: Option<i64>,
    ) -> Result<(), Error> {
        let mut lease_id = 0;
        let final_key = self.prefix_namespace(key);
        if let Some(ttl_param) = ttl {
            let response = self.client.lease_grant(ttl_param, None).await?;
            lease_id = response.id()
        } else {
            let response = self
                .client
                .kv_get_request(final_key.clone(), Some(GetOptions::new().with_keys_only()))
                .await?;
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

        self.client
            .kv_put_request(final_key, value.into(), option)
            .await?;

        Ok(())
    }

    /// 将Key绑定到lease中
    pub async fn kv_put_with_lease(
        &mut self,
        key: impl Into<Vec<u8>>,
        value: impl Into<Vec<u8>>,
        lease: i64,
    ) -> Result<(), Error> {
        let final_key = self.prefix_namespace(key);
        self.client
            .kv_put_request(
                final_key,
                value.into(),
                Some(PutOptions::new().with_lease(lease)),
            )
            .await?;

        Ok(())
    }

    /// 删除键值对
    pub async fn kv_delete(&mut self, keys: Vec<impl Into<Vec<u8>>>) -> Result<usize, Error> {
        let mut success = 0usize;
        for key in keys {
            let result = self
                .client
                .kv_delete_request(self.prefix_namespace(key), None)
                .await;
            if result.is_ok() {
                success += 1;
            }
        }

        Ok(success)
    }

    /// 获取某一个key的历史版本，如果中间某个版本以及被删除或压缩，将终止搜索
    pub async fn kv_get_history_versions(
        &mut self,
        key: impl Into<Vec<u8>>,
        start: i64,
        end: i64,
    ) -> Result<Vec<i64>, Error> {
        let mut history = Vec::new();
        let final_key = self.prefix_namespace(key);
        self.kv_get_history_versions0(final_key, end, start, end, &mut history)
            .await;
        Ok(history)
    }

    fn kv_get_history_versions0<'a>(
        &'a mut self,
        key: Vec<u8>,
        revision: i64,
        start: i64,
        end: i64,
        history: &'a mut Vec<i64>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            if revision >= start && revision <= end {
                let result = self
                    .client
                    .kv_get_request(
                        key.clone(),
                        Some(GetOptions::new().with_keys_only().with_revision(revision)),
                    )
                    .await;
                match result {
                    Ok(response) => {
                        let kvs = response.kvs();
                        if kvs.is_empty() {
                            return ();
                        }
                        let kv = &kvs[0];
                        let next_revision =
                            if revision >= kv.create_revision() && revision <= kv.mod_revision() {
                                history.push(revision);
                                revision - 1
                            } else {
                                kv.mod_revision()
                            };
                        self.kv_get_history_versions0(key, next_revision, start, end, history)
                            .await;
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

    fn root_key(&self) -> Vec<u8> {
        self.get_namespace_unchecked().clone().into_bytes()
    }

    fn prefix_namespace(&self, end_key: impl Into<Vec<u8>>) -> Vec<u8> {
        if self.has_namespace() {
            let mut full_key = self.root_key();
            full_key.append(&mut end_key.into());
            full_key
        } else {
            end_key.into()
        }
    }

    fn prefix_namespace_to_range_end(&self, end_key: impl Into<Vec<u8>>) -> Vec<u8> {
        if self.has_namespace() {
            let mut end_key_bytes: Vec<u8> = end_key.into();
            let mut prefix = self.get_namespace_unchecked().clone().into_bytes();
            // range end is '\0', calculate the prefixed range end by (key + 1)
            if end_key_bytes.len() == 1 && end_key_bytes[0] == 0 {
                key_next(&mut prefix);
                prefix
            } else {
                prefix.append(&mut end_key_bytes);
                prefix
            }
        } else {
            end_key.into()
        }
    }

    /// 获取所有lease id
    pub async fn leases(&mut self) -> Result<Vec<String>, Error> {
        let response = self.client.leases().await?;
        let mut leases = Vec::new();
        for lease in response.leases() {
            leases.push(lease.id().to_string())
        }
        Ok(leases)
    }

    /// 获取lease的详情信息
    pub async fn lease_get(&mut self, lease: i64) -> Result<SerializableLeaseInfo, Error> {
        let response = self
            .client
            .lease_time_to_live(lease, Some(LeaseTimeToLiveOptions::new().with_keys()))
            .await?;
        let ttl = response.ttl();
        let granted_ttl = response.granted_ttl();
        let id = response.id().to_string();
        let bind_keys = response.keys();
        let mut keys = Vec::with_capacity(bind_keys.len());
        for bind_key in bind_keys {
            let s = String::from_utf8_lossy(bind_key.as_slice()).to_string();
            keys.push(s);
        }

        Ok(SerializableLeaseInfo {
            id,
            ttl,
            granted_ttl,
            keys,
        })
    }

    /// 获取lease的简要详情信息
    pub async fn lease_get_simple_info(
        &mut self,
        lease: i64,
    ) -> Result<SerializableLeaseSimpleInfo, Error> {
        let response = self
            .client
            .lease_time_to_live(lease, Some(LeaseTimeToLiveOptions::new().with_keys()))
            .await?;
        let ttl = response.ttl();
        let granted_ttl = response.granted_ttl();

        Ok(SerializableLeaseSimpleInfo { ttl, granted_ttl })
    }

    /// 授权新的lease或为已存在的lease续租
    pub async fn lease_grant(&mut self, ttl: i64, lease: Option<i64>) -> Result<i64, Error> {
        let options = lease.map(|id| LeaseGrantOptions::new().with_id(id));
        let response = self.client.lease_grant(ttl, options).await?;
        Ok(response.id())
    }

    /// 回收lease
    pub async fn lease_revoke(&mut self, lease: i64) -> Result<(), Error> {
        self.client.lease_revoke(lease).await?;
        Ok(())
    }

    /// 查询所有用户
    pub async fn user_list(&mut self) -> Result<Vec<SerializableUser>, Error> {
        let response = self.client.user_list().await?;
        let mut auth_client = self.client.get_inner().auth_client();
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
    pub async fn user_add(&mut self, user: String, password: String) -> Result<(), Error> {
        self.client.user_add(user, password, None).await?;
        Ok(())
    }

    /// 删除用户
    pub async fn user_delete(&mut self, user: String) -> Result<(), Error> {
        self.client.user_delete(user).await?;
        Ok(())
    }

    /// 修改用户密码
    pub async fn user_change_password(
        &mut self,
        user: String,
        password: String,
    ) -> Result<(), Error> {
        self.client.user_change_password(user, password).await?;
        Ok(())
    }

    /// 给用户授权角色
    pub async fn user_grant_role(&mut self, user: String, role: String) -> Result<(), Error> {
        self.client.user_grant_role(user, role).await?;
        Ok(())
    }

    /// 回收用户的角色
    pub async fn user_revoke_role(&mut self, user: String, role: String) -> Result<(), Error> {
        self.client.user_revoke_role(user, role).await?;
        Ok(())
    }

    /// 判断用户是否是 root 用户（拥有root角色权限的用户也被认为是root用户）
    pub async fn user_is_root(&mut self, user: &String) -> Result<bool, Error> {
        if user == "root" {
            return Ok(true);
        }

        let response = self.client.user_get(user).await?;
        for role in response.roles() {
            if role == "root" {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// 开启权限验证功能，此功能调用后可能会导致connector无法使用
    pub async fn auth_enable(&mut self) -> Result<(), Error> {
        self.client.auth_enable().await?;
        Ok(())
    }

    /// 关闭权限验证功能，此功能调用后可能会导致connector无法使用
    pub async fn auth_disable(&mut self) -> Result<(), Error> {
        self.client.auth_disable().await?;
        Ok(())
    }

    /// 获取所有角色
    pub async fn role_list(&mut self) -> Result<Vec<String>, Error> {
        let response = self.client.role_list().await?;
        Ok(Vec::from(response.roles()))
    }

    /// 获取角色的权限信息
    pub async fn role_get_permissions(
        &mut self,
        role: String,
    ) -> Result<Vec<SerializablePermission>, Error> {
        let response = self.client.role_get(role).await?;
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
            let all_keys = (key_bytes_len == 0
                && (range_end_len == 0 || (range_end_len == 1 && range_end[0] == 0)))
                || (key_bytes_len == 1
                    && range_end_len == 1
                    && key_bytes[0] == 0
                    && range_end[0] == 0);

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
    pub async fn role_add(&mut self, role: String) -> Result<(), Error> {
        self.client.role_add(role).await?;
        Ok(())
    }

    /// 删除角色
    pub async fn role_delete(&mut self, role: String) -> Result<(), Error> {
        self.client.role_delete(role).await?;
        Ok(())
    }

    /// 给角色授权权限
    pub async fn role_grant_permission(
        &mut self,
        role: String,
        permission: SerializablePermission,
    ) -> Result<(), Error> {
        self.client
            .role_grant_permission(role, permission.into())
            .await?;
        Ok(())
    }

    /// 回收角色的权限
    pub async fn role_revoke_permission(
        &mut self,
        role: String,
        permission: SerializablePermission,
    ) -> Result<(), Error> {
        let range_ned = permission.parse_range_end();
        self.client
            .role_revoke_permission(
                role,
                permission.key,
                Some(RoleRevokePermissionOptions::new().with_range_end(range_ned)),
            )
            .await?;

        Ok(())
    }

    /// 获取集群的详情信息，包含集群数据、成员、报警、状态等信息
    pub async fn cluster_get(&mut self) -> Result<SerializableCluster, Error> {
        let mut response = self.client.member_list().await?;
        let status = self.client.status().await?;

        let cluster_status = SerializableClusterStatus {
            version: String::from(status.version()),
            db_size_allocated: status.db_size(),
            db_size_used: status.raft_used_db_size(),
            leader: status.leader().to_string(),
            raft_index: status.raft_index().to_string(),
            raft_term: status.raft_term().to_string(),
            raft_applied_index: status.raft_applied_index().to_string(),
            errors: Vec::from(status.errors()),
        };
        let alarm_response = self
            .client
            .alarm(AlarmAction::Get, AlarmType::None, None)
            .await?;
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
                alarm_type: *alarms_map
                    .get(&member.id())
                    .unwrap_or_else(|| &AlarmType::None) as i32,
            })
        }

        let header = response.take_header().unwrap();

        Ok(SerializableCluster {
            id: header.cluster_id().to_string(),
            member_id: header.member_id().to_string(),
            revision: header.revision(),
            members,
            status: cluster_status,
        })
    }

    /// 集群添加新成员节点
    pub async fn cluster_add_member(&mut self, urls: impl Into<Vec<String>>) -> Result<(), Error> {
        self.client.member_add(urls.into(), None).await?;
        Ok(())
    }

    /// 集群移除成员节点
    pub async fn cluster_remove_member(&mut self, id: String) -> Result<(), Error> {
        match id.parse::<u64>() {
            Ok(id) => {
                self.client.member_remove(id).await?;
                Ok(())
            }
            Err(e) => Err(Error::InvalidArgs(e.to_string())),
        }
    }

    /// 集群更新成员节点
    pub async fn cluster_update_member(
        &mut self,
        id: String,
        urls: impl Into<Vec<String>>,
    ) -> Result<(), Error> {
        match id.parse::<u64>() {
            Ok(id) => {
                self.client.member_update(id, urls.into()).await?;
                Ok(())
            }
            Err(e) => Err(Error::InvalidArgs(e.to_string())),
        }
    }

    /// 对节点进行碎片整理。这是一个比较消耗资源的操作，谨慎调用。
    pub async fn maintenance_defragment(&mut self) -> Result<(), Error> {
        self.client.defragment().await?;
        Ok(())
    }

    /// 保存数据快照
    pub async fn maintenance_snapshot(
        &mut self,
        file_path: PathBuf,
        watch_sender: mpsc::Sender<(u64, u64, Option<String>)>,
        stop_receiver: oneshot::Receiver<()>,
    ) -> Result<(), LogicError> {
        let mut stream = self.client.snapshot().await?;
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        if file_path.exists() {
            fs::remove_file(&file_path)?;
        }

        File::create(&file_path).await?;
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(file_path)
            .await?;

        let watch_task = async move {
            loop {
                let slip_result = stream.message().await;
                match slip_result {
                    Ok(slip) => {
                        if let Some(response) = slip {
                            let blob = response.blob();
                            let remain = response.remaining_bytes();

                            let received = blob.len() as u64;

                            file.write_all(blob).await;

                            debug!("snapshot [remain] {}-{}", received, remain);
                            watch_sender
                                .send((received, remain, None))
                                .await
                                .unwrap_or_else(|e| error!("watch send error[remain]: {e}"));
                            if remain == 0 {
                                break;
                            }
                        } else {
                            watch_sender
                                .send((0, 0, None))
                                .await
                                .unwrap_or_else(|e| error!("watch send error[finish]: {e}"));
                            break;
                        }
                    }
                    Err(e) => {
                        watch_sender
                            .send((0, 0, Some(e.to_string())))
                            .await
                            .unwrap_or_else(|e| error!("watch send error[failed]: {e}"));
                        debug!("snapshot [failed]: {e}");
                        break;
                    }
                }
            }
        };

        let task = tokio::spawn(async {
            select! {
                _ = watch_task => {
                    info!("Snapshot task finished")
                },
                _ = stop_receiver => {
                    info!("Snapshot task stopped")
                }
            }
        });

        Ok(())
    }
}

fn key_next(key: &mut Vec<u8>) {
    let len = key.len();
    if key[len - 1] == u8::MAX {
        key.push(0u8)
    } else {
        key[len - 1] += 1
    }
}
pub struct SnapshotTask {
    pub name: String,
    pub folder: String,
    pub state: SnapshotState,
    pub stop_notifier: Option<oneshot::Sender<()>>,
}
