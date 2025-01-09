use log::debug;
use prost::Message;

use crate::{proto::k8s_io_api_core_v1::Pod, transport::kv::{FormattedValue, FormatKind}};

/// 尝试格式化k8s存储的值
/// 
/// - `key`: 全路径
/// - `value`: 值内容
/// 
/// 如果格式化成功，将返回一个Json字符串，表达为 由`Option`包裹的`FormattedValue`。
/// 
/// 如果格式化失败，返回 `None`
pub fn try_format_value(key: &String, value: &Vec<u8>) -> Option<FormattedValue> {
    if key.starts_with("/registry/pods/") {
        try_format_pods(value)
    } else if key.starts_with("/registry/services/") {
        try_format_services(value)
    } else if key.starts_with("/registry/deployments/") {
        try_format_deployments(value)
    } else if key.starts_with("/registry/clusterroles/") {
        try_format_clusterroles(value)
    } else if key.starts_with("/registry/configmaps/") {
        try_format_configmaps(value)
    } else if key.starts_with("/registry/controllerrevisions/") {
        try_format_controllerrevisions(value)
    } else if key.starts_with("/registry/csinodes/") {
        try_format_csinodes(value)
    } else {
        None
    }
}

/// parse from `k8s.io.api.core.v1#Pod`
fn try_format_pods(value: &Vec<u8>) -> Option<FormattedValue> {
    let result = Pod::decode(value.as_slice());
    match result {
        Ok(pod) => {
            let json_result = serde_json::to_string(&pod);
            match json_result {
                Ok(s) => {
                    debug!("pod: {}", s);
                    return Some(FormattedValue{
                        kind: FormatKind::Json,
                        value: s
                    })
                },
                Err(e) => {
                    debug!("serde_json::to_string(pod) failed: {e}");
                }
            }
        },
        Err(e) => {
            debug!("decode k8s.io.api.core.v1#Pod failed: {e}");
        }
    }
    None
}

/// parse from `k8s.io.api.core.v1#Service`
fn try_format_services(value: &Vec<u8>) -> Option<FormattedValue> {
    None
}

/// parse from `k8s.io.api.apps.v1#Deployment`
fn try_format_deployments(value: &Vec<u8>) -> Option<FormattedValue> {
    None
}

/// parse from `k8s.io.api.rbac.v1#ClusterRole`
fn try_format_clusterroles(value: &Vec<u8>) -> Option<FormattedValue> {
    None
}

/// parse from `k8s.io.api.core.v1#ConfigMap`
fn try_format_configmaps(value: &Vec<u8>) -> Option<FormattedValue> {
    None
}

/// parse from `k8s.io.api.apps.v1#ControllerRevision`
fn try_format_controllerrevisions(value: &Vec<u8>) -> Option<FormattedValue> {
    None
}

/// parse from `k8s.io.api.storage.v1#CSINode`
fn try_format_csinodes(value: &Vec<u8>) -> Option<FormattedValue> {
    None
}