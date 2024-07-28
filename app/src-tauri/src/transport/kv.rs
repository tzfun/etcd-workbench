use etcd_client::KeyValue;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializableKeyValue {
    pub key: String,
    pub create_revision: i64,
    pub mod_revision: i64,
    pub version: i64,
    pub value: Vec<u8>,
    pub lease: i64,
}

impl From<KeyValue> for SerializableKeyValue {
    fn from(kv: KeyValue) -> Self {
        unsafe {
            let key = String::from(kv.key_str_unchecked());
            let value = Vec::from(kv.value());
            let create_revision = kv.create_revision();
            let mod_revision = kv.mod_revision();
            let version = kv.version();
            let lease = kv.lease();
            SerializableKeyValue {
                key,
                value,
                create_revision,
                mod_revision,
                version,
                lease,
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializableKeyValues {
    inner: Vec<SerializableKeyValue>,
}

impl SerializableKeyValues {
    pub fn from_kv_vec(kv_vec: Vec<KeyValue>) -> SerializableKeyValues {
        let mut kvs = Vec::with_capacity(kv_vec.len());
        for kv in kv_vec {
            kvs.push(SerializableKeyValue::from(kv));
        }
        SerializableKeyValues {
            inner: kvs
        }
    }

    pub fn take(self) -> Vec<SerializableKeyValue> {
        self.inner
    }
}
