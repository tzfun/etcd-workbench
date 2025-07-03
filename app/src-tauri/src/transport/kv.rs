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
        //  全路径，包含namespace
        let key = String::from_utf8_lossy(kv.key()).to_string();
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

    /// 从集合中转换，并从key中移除namespace
    pub fn from_vec(kvs: Vec<KeyValue>, namespace: Option<&String>) -> Vec<SerializableKeyValue> {
        let mut arr = Vec::with_capacity(kvs.len());
        for kv in kvs {
            let mut s_kv = SerializableKeyValue::from(kv);
            if let Some(namespace) = namespace {
                s_kv.remove_prefix(namespace);
            }
            arr.push(s_kv);
        }
        arr
    }
}

impl From<KeyValue> for SerializableKeyValue {
    fn from(kv: KeyValue) -> Self {
        Self::from_ref(&kv)
    }
}

impl SerializableKeyValue {
    pub fn remove_prefix(&mut self, prefix: &String) {
        self.key_bytes.drain(0..prefix.as_bytes().len());
        self.key = String::from_utf8_lossy(&self.key_bytes).to_string();
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