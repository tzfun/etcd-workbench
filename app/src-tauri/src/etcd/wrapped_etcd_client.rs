use etcd_client::{
    AlarmAction, AlarmOptions, AlarmResponse, AlarmType, AuthDisableResponse, AuthEnableResponse, DefragmentResponse, DeleteOptions, DeleteResponse, GetOptions, GetResponse, LeaseGrantOptions, LeaseGrantResponse, LeaseLeasesResponse, LeaseRevokeResponse, LeaseTimeToLiveOptions, LeaseTimeToLiveResponse, MemberAddOptions, MemberAddResponse, MemberListResponse, MemberRemoveResponse, MemberUpdateResponse, Permission, PutOptions, PutResponse, RoleAddResponse, RoleDeleteResponse, RoleGetResponse, RoleGrantPermissionResponse, RoleListResponse, RoleRevokePermissionOptions, RoleRevokePermissionResponse, SnapshotStreaming, StatusResponse, UserAddOptions, UserAddResponse, UserChangePasswordResponse, UserDeleteResponse, UserGetResponse, UserGrantRoleResponse, UserListResponse, UserRevokeRoleResponse
};

use crate::transport::connection::ConnectionUser;

#[derive(Clone)]
pub struct WrappedEtcdClient {
    inner: etcd_client::Client,
    auth: Option<ConnectionUser>,
}

impl WrappedEtcdClient {
    pub fn new(client: etcd_client::Client, auth: Option<ConnectionUser>) -> Self {
        WrappedEtcdClient {
            inner: client,
            auth,
        }
    }

    pub async fn authenticate(&mut self) -> Result<(), etcd_client::Error> {
        let auth = self.auth.clone();
        if let Some(user) = auth {
            self.inner
                .set_client_auth(user.username, user.password)
                .await
        } else {
            Ok(())
        }
    }

    pub fn get_inner(&self) -> &etcd_client::Client {
        &self.inner
    }

    pub async fn kv_get_request(
        &mut self,
        key: Vec<u8>,
        option: Option<GetOptions>,
    ) -> Result<GetResponse, etcd_client::Error> {
        let result = self.inner.get(key.clone(), option.clone()).await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.get(key, option).await;
                }
            }
        }
        result
    }

    pub async fn kv_put_request(
        &mut self,
        key: Vec<u8>,
        value: Vec<u8>,
        option: Option<PutOptions>,
    ) -> Result<PutResponse, etcd_client::Error> {
        let result = self
            .inner
            .put(key.clone(), value.clone(), option.clone())
            .await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.put(key, value, option).await;
                }
            }
        }
        result
    }

    pub async fn kv_delete_request(
        &mut self,
        key: Vec<u8>,
        option: Option<DeleteOptions>,
    ) -> Result<DeleteResponse, etcd_client::Error> {
        let result = self.inner.delete(key.clone(), option.clone()).await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.delete(key, option).await;
                }
            }
        }
        result
    }

    pub async fn leases(&mut self) -> Result<LeaseLeasesResponse, etcd_client::Error> {
        let result = self.inner.leases().await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.leases().await;
                }
            }
        }
        result
    }

    pub async fn lease_grant(
        &mut self,
        ttl: i64,
        option: Option<LeaseGrantOptions>,
    ) -> Result<LeaseGrantResponse, etcd_client::Error> {
        let result = self.inner.lease_grant(ttl.clone(), option.clone()).await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.lease_grant(ttl, option).await;
                }
            }
        }
        result
    }

    pub async fn lease_revoke(
        &mut self,
        id: i64,
    ) -> Result<LeaseRevokeResponse, etcd_client::Error> {
        let result = self.inner.lease_revoke(id).await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.lease_revoke(id).await;
                }
            }
        }
        result
    }

    pub async fn lease_time_to_live(
        &mut self,
        id: i64,
        option: Option<LeaseTimeToLiveOptions>,
    ) -> Result<LeaseTimeToLiveResponse, etcd_client::Error> {
        let result = self.inner.lease_time_to_live(id, option.clone()).await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.lease_time_to_live(id, option).await;
                }
            }
        }
        result
    }

    pub async fn user_list(&mut self) -> Result<UserListResponse, etcd_client::Error> {
        let result = self.inner.user_list().await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.user_list().await;
                }
            }
        }
        result
    }

    pub async fn user_add(
        &mut self,
        name: String,
        password: String,
        options: Option<UserAddOptions>,
    ) -> Result<UserAddResponse, etcd_client::Error> {
        let result = self
            .inner
            .user_add(name.clone(), password.clone(), options.clone())
            .await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.user_add(name, password, options).await;
                }
            }
        }
        result
    }

    pub async fn user_delete(
        &mut self,
        user: String,
    ) -> Result<UserDeleteResponse, etcd_client::Error> {
        let result = self.inner.user_delete(user.clone()).await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.user_delete(user).await;
                }
            }
        }
        result
    }

    pub async fn user_change_password(
        &mut self,
        user: String,
        password: String,
    ) -> Result<UserChangePasswordResponse, etcd_client::Error> {
        let result = self
            .inner
            .user_change_password(user.clone(), password.clone())
            .await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.user_change_password(user, password).await;
                }
            }
        }
        result
    }

    pub async fn user_grant_role(
        &mut self,
        user: String,
        role: String,
    ) -> Result<UserGrantRoleResponse, etcd_client::Error> {
        let result = self.inner.user_grant_role(user.clone(), role.clone()).await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.user_grant_role(user, role).await;
                }
            }
        }
        result
    }

    pub async fn user_revoke_role(
        &mut self,
        user: String,
        role: String,
    ) -> Result<UserRevokeRoleResponse, etcd_client::Error> {
        let result = self
            .inner
            .user_revoke_role(user.clone(), role.clone())
            .await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.user_revoke_role(user, role).await;
                }
            }
        }
        result
    }

    pub async fn user_get(&mut self, user: &String) -> Result<UserGetResponse, etcd_client::Error> {
        let result = self.inner.user_get(user.clone()).await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.user_get(user).await;
                }
            }
        }
        result
    }

    pub async fn auth_enable(&mut self) -> Result<AuthEnableResponse, etcd_client::Error> {
        let result = self.inner.auth_enable().await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.auth_enable().await;
                }
            }
        }
        result
    }

    pub async fn auth_disable(&mut self) -> Result<AuthDisableResponse, etcd_client::Error> {
        let result = self.inner.auth_disable().await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.auth_disable().await;
                }
            }
        }
        result
    }

    pub async fn role_list(&mut self) -> Result<RoleListResponse, etcd_client::Error> {
        let result = self.inner.role_list().await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.role_list().await;
                }
            }
        }
        result
    }

    pub async fn role_get(&mut self, role: String) -> Result<RoleGetResponse, etcd_client::Error> {
        let result = self.inner.role_get(role.clone()).await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.role_get(role).await;
                }
            }
        }
        result
    }

    pub async fn role_add(&mut self, role: String) -> Result<RoleAddResponse, etcd_client::Error> {
        let result = self.inner.role_add(role.clone()).await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.role_add(role).await;
                }
            }
        }
        result
    }

    pub async fn role_delete(
        &mut self,
        role: String,
    ) -> Result<RoleDeleteResponse, etcd_client::Error> {
        let result = self.inner.role_delete(role.clone()).await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.role_delete(role).await;
                }
            }
        }
        result
    }

    pub async fn role_grant_permission(
        &mut self,
        role: String,
        permission: Permission,
    ) -> Result<RoleGrantPermissionResponse, etcd_client::Error> {
        let result = self
            .inner
            .role_grant_permission(role.clone(), permission.clone())
            .await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.role_grant_permission(role, permission).await;
                }
            }
        }
        result
    }

    pub async fn role_revoke_permission(
        &mut self,
        role: String,
        key: String,
        option: Option<RoleRevokePermissionOptions>,
    ) -> Result<RoleRevokePermissionResponse, etcd_client::Error> {
        let result = self
            .inner
            .role_revoke_permission(role.clone(), key.clone(), option.clone())
            .await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.role_revoke_permission(role, key, option).await;
                }
            }
        }
        result
    }

    pub async fn member_list(&mut self) -> Result<MemberListResponse, etcd_client::Error> {
        let result = self.inner.member_list().await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.member_list().await;
                }
            }
        }
        result
    }

    pub async fn status(&mut self) -> Result<StatusResponse, etcd_client::Error> {
        let result = self.inner.status().await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.status().await;
                }
            }
        }
        result
    }

    pub async fn alarm(
        &mut self,
        alarm_action: AlarmAction,
        alarm_type: AlarmType,
        option: Option<AlarmOptions>,
    ) -> Result<AlarmResponse, etcd_client::Error> {
        let result = self
            .inner
            .alarm(alarm_action.clone(), alarm_type.clone(), option.clone())
            .await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.alarm(alarm_action, alarm_type, option).await;
                }
            }
        }
        result
    }

    pub async fn member_add(
        &mut self,
        urls: Vec<String>,
        option: Option<MemberAddOptions>,
    ) -> Result<MemberAddResponse, etcd_client::Error> {
        let result = self.inner.member_add(urls.clone(), option.clone()).await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.member_add(urls, option).await;
                }
            }
        }
        result
    }

    pub async fn member_remove(
        &mut self,
        id: u64
    ) -> Result<MemberRemoveResponse, etcd_client::Error> {
        let result = self.inner.member_remove(id).await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.member_remove(id).await;
                }
            }
        }
        result
    }

    pub async fn member_update(
        &mut self,
        id: u64,
        url: Vec<String>
    ) -> Result<MemberUpdateResponse, etcd_client::Error> {
        let result = self.inner.member_update(id, url.clone()).await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.member_update(id, url).await;
                }
            }
        }
        result
    }
    
    pub async fn defragment(&mut self) -> Result<DefragmentResponse, etcd_client::Error> {
        let result = self.inner.defragment().await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.defragment().await;
                }
            }
        }
        result
    }

    pub async fn snapshot(&mut self) -> Result<SnapshotStreaming, etcd_client::Error> {
        let result = self.inner.snapshot().await;

        if let Err(etcd_client::Error::GRpcStatus(s)) = &result {
            if s.code() as i32 == 16 {
                let self_auth = self.auth.clone();
                if let Some(auth) = self_auth {
                    self.authenticate().await?;
                    return self.inner.snapshot().await;
                }
            }
        }
        result
    }
}
