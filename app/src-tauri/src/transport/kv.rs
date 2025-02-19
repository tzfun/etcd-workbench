use etcd_client::KeyValue;
use serde::{Deserialize, Serialize};


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
    pub create_revision: i64,
    pub mod_revision: i64,
    pub version: i64,
    pub value: Vec<u8>,
    pub lease: String,
    pub lease_info: Option<SerializableLeaseSimpleInfo>,
    pub formatted_value: Option<FormattedValue>
}

impl From<KeyValue> for SerializableKeyValue {
    fn from(kv: KeyValue) -> Self {
        unsafe {
            let key = String::from(kv.key_str_unchecked());
            let value = Vec::from(kv.value());
            let create_revision = kv.create_revision();
            let mod_revision = kv.mod_revision();
            let version = kv.version();
            let lease = kv.lease().to_string();
            SerializableKeyValue {
                key,
                value,
                create_revision,
                mod_revision,
                version,
                lease,
                lease_info: None,
                formatted_value: None
            }
        }
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
    pub exist_value: Option<Vec<u8>>,
    pub exist_version: Option<i64>
}