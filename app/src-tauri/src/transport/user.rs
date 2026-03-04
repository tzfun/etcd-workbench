use etcd_client::{Permission, PermissionType};
use log::warn;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializableUser {
    pub user: String,
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct SerializablePermission {
    /// 全路径，包含 namespace
    pub key: String,
    /// 全路径，包含 namespace
    #[serde(default)]
    pub key_bytes: Vec<u8>,
    /// permission::Type
    pub perm_type: i32,
    pub prefix: bool,
    pub all_keys: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct ReadableKeys {
    /// 是否可以读当前 namespace 下全部key 可读的keys
    pub read_all_keys: bool,
    /// 如果不可以读全部可以，此字段存可以读的前缀key集合
    /// 
    /// key均为全路径，包含 namespace
    pub prefix_keys: Option<Vec<Vec<u8>>>,
    /// 如果不可以读全部可以，此字段存可以读的key全路径
    ///
    /// key均为全路径，包含 namespace
    pub full_path_keys: Option<Vec<Vec<u8>>>,
}

impl Into<Permission> for SerializablePermission {
    fn into(self) -> Permission {
        let range_end = self.parse_range_end();
        let perm_type = PermissionType::try_from(self.perm_type).unwrap_or_else(|p| {
            warn!("Catch a unknown enum value in PermissionType: {}", p);
            PermissionType::Read
        });
        let permission = Permission::new(perm_type, self.key);
        permission.with_range_end(range_end)
    }
}

impl SerializablePermission {
    pub fn parse_range_end(&self) -> Vec<u8> {
        if self.all_keys {
            vec![b'\0']
        } else if self.prefix {
            let mut range_end = Vec::from(self.key.as_bytes());
            let len = range_end.len();
            if len > 0 {
                let last = range_end.last().unwrap();
                if *last == u8::MAX {
                    range_end.push(1);
                } else {
                    range_end[len - 1] += 1;
                }
            }

            range_end
        } else {
            vec![]
        }
    }
}