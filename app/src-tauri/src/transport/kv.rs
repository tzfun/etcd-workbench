use etcd_client::KeyValue;
use serde::{Deserialize, Serialize};

use crate::utils::k8s_formatter;


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all="camelCase")]
pub enum FormatLanguage {
    Json
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all="camelCase")]
pub enum FormatSource {
    Kubernetes
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all="camelCase")]
pub struct FormattedValue {
    pub source: FormatSource,
    //  格式化类型
    pub language: FormatLanguage,
    //  格式化内容
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all="camelCase")]
pub struct SerializableKeyValue {
    pub key: String,
    pub key_bytes: Vec<u8>,
    pub key_encoded_utf8: bool,
    pub create_revision: i64,
    pub mod_revision: i64,
    pub version: i64,
    pub value: Vec<u8>,
    pub lease: String,
    pub lease_info: Option<SerializableLeaseSimpleInfo>,
    pub formatted_value: Option<FormattedValue>
}

impl SerializableKeyValue {
    pub fn from_ref(kv: &KeyValue) -> Self {
        unsafe {
            let key = String::from(kv.key_str_unchecked());
            let key_bytes = Vec::from(kv.key());
            let key_encoded_utf8 = std::str::from_utf8(kv.key()).is_ok();
            let value = Vec::from(kv.value());
            let create_revision = kv.create_revision();
            let mod_revision = kv.mod_revision();
            let version = kv.version();
            let lease = kv.lease().to_string();
            let formatted_value = k8s_formatter::try_format_proto(&key, &value);
            SerializableKeyValue {
                key,
                key_bytes,
                key_encoded_utf8,
                value,
                create_revision,
                mod_revision,
                version,
                lease,
                lease_info: None,
                formatted_value,
            }
        }
    }
}

impl From<KeyValue> for SerializableKeyValue {
    fn from(kv: KeyValue) -> Self {
        Self::from_ref(&kv)
    }
}

impl SerializableKeyValue {
    pub fn remove_prefix(&mut self, prefix: &String) {
        self.key = self.key.replace(prefix.as_str(), "")
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct SerializableLeaseInfo {
    pub id: String,
    pub ttl: i64,
    pub granted_ttl: i64,
    pub keys: Vec<String>
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all="camelCase")]
pub struct SerializableLeaseSimpleInfo {
    pub ttl: i64,
    pub granted_ttl: i64
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all="camelCase")]
pub struct SearchResult {
    pub count: usize,
    pub results: Vec<SerializableKeyValue>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all="camelCase")]
pub struct KVPutResult {
    pub success: bool,
    pub final_kv: Option<SerializableKeyValue>,
    pub exist_value: Option<Vec<u8>>,
    pub exist_version: Option<i64>
}