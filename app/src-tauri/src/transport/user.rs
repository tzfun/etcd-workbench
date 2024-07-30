use etcd_client::{Permission, PermissionType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializableUser {
    pub user: String,
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializablePermission {
    pub key: String,
    pub perm_type: PermissionType,
    pub prefix: bool,
    pub all_keys: bool,
}

impl Into<Permission> for SerializablePermission {
    fn into(self) -> Permission {
        let range_end = self.parse_range_end();
        let mut permission = Permission::new(self.perm_type, self.key);
        permission.with_range_end(range_end)
    }
}

impl SerializablePermission {
    pub fn parse_range_end(&self) -> Vec<u8> {
        if self.all_keys {
            vec![b'\0']
        } else if self.prefix {
            let mut range_end = Vec::from(self.key.as_bytes());
            if !range_end.is_empty() {
                let last = range_end.last().unwrap();
                if *last == u8::MAX_VALUE {
                    range_end.push(1);
                } else {
                    *range_end[range_end.len() - 1] += 1;
                }
            }

            range_end
        } else {
            vec![]
        }
    }
}