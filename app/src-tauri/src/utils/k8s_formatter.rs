use log::debug;
use prost::{DecodeError, Message};
use serde::Serialize;

use crate::{
    proto::{self, k8s::io::apimachinery::pkg::runtime::Unknown},
    transport::kv::{FormatLanguage, FormatSource, FormattedValue},
};

const PROTO_PREFIX: [u8; 4] = [0x6b, 0x38, 0x73, 0x00];

/// 尝试格式化k8s存储的值
///
/// - `key`: 全路径
/// - `value`: 值内容
///
/// 如果格式化成功，将返回一个Json字符串，表达为 由`Option`包裹的`FormattedValue`。
///
/// 如果格式化失败，返回 `None`
pub fn try_format_proto(key: &String, value: &Vec<u8>) -> Option<FormattedValue> {
    if key.starts_with("/registry") {
        return try_format(value, |kind, version, raw| {
            if version.eq("v1") {
                return try_format_core_v1(kind, version, raw);
            } else if version.eq("apps/v1") {
                return try_format_apps_v1(kind, version, raw);
            } else if version.eq("rbac.authorization.k8s.io/v1") {
                return try_format_rbac_v1(kind, version, raw);
            } else if version.eq("storage.k8s.io/v1") {
                return try_format_storage_v1(kind, version, raw);
            } else if version.eq("discovery.k8s.io/v1") {
                // Note: 虽然有多个版本，但互相不兼容，因此需各个版本单独处理
                return try_format_discovery_v1(kind, version, raw);
            } else if version.eq("discovery.k8s.io/v1beta1") {
                // Note: 虽然有多个版本，但互相不兼容，因此需各个版本单独处理
                return try_format_discovery_v1beta1(kind, version, raw);
            } else if version.starts_with("flowcontrol.apiserver.k8s.io/v1") {
                // Note: 虽然有多个版本（v1beta1, v1beta2, v1beta3），但其中并没有实质性的修改。
                // 均是对注释修改或者是字段重命名，因此用v1版本解析即可。
                return try_format_flowcontrol_v1(kind, version, raw);
            } else if version.eq("coordination.k8s.io/v1") {
                return try_format_coordination_v1(kind, version, raw);
            } else if version.eq("scheduling.k8s.io/v1") {
                return try_format_scheduling_v1(kind, version, raw);
            } else if version.starts_with("batch/v1") {
                // Note: 虽然有多个版本，但互相不兼容，因此需各个版本单独处理
                return try_format_batch_v1(kind, version, raw);
            } else if version.starts_with("networking.k8s.io/v1") {
                // Note: 虽然有多个版本，但互相不兼容，因此需各个版本单独处理
                return try_format_networking_v1(kind, version, raw);
            } else if version.starts_with("admissionregistration.k8s.io/v1") {
                // Note: 虽然有多个版本，但互相不兼容，因此需各个版本单独处理
                return try_format_admissionregistration_v1(kind, version, raw);
            }
            None
        });
    }
    None
}

fn try_format_unknown(value: &Vec<u8>) -> Option<Unknown> {
    let prefix_len = PROTO_PREFIX.len();
    let value_len = value.len();
    if prefix_len >= value_len {
        return None;
    }
    let new_value = value[prefix_len..].to_vec();

    let result = Unknown::decode(new_value.as_slice());
    match result {
        Ok(unknown) => {
            return Some(unknown);
        }
        Err(e) => {
            debug!("decode k8s.io.apimachinery.pkg.runtime#Unknown failed: {e}");
        }
    }
    None
}

fn try_format_to_json<T>(value: &T) -> Option<FormattedValue>
where
    T: ?Sized + Serialize,
{
    let json_result = serde_json::to_string_pretty(value);
    match json_result {
        Ok(s) => {
            return Some(FormattedValue {
                source: FormatSource::Kubernetes,
                language: FormatLanguage::Json,
                value: s,
            });
        }
        Err(e) => {
            debug!("serde_json::to_string() failed: {e}");
        }
    }
    None
}

fn try_format<F>(value: &Vec<u8>, f: F) -> Option<FormattedValue>
where
    F: Fn(&str, &str, &[u8]) -> Option<FormattedValue>,
{
    let result = try_format_unknown(value);
    if let Some(unknown) = result {
        if let Some(type_meta) = &unknown.type_meta {
            let api_version = type_meta.api_version();
            let kind = type_meta.kind();

            let v = f(kind, api_version, unknown.raw());
            if v.is_none() {
                debug!("Kubernetes decoder missed: {} {}", kind, api_version);
            }
            return v;
        }
    }

    None
}

fn decode_err_handle(kind: &str, version: &str, e: DecodeError) {
    debug!("Kubernetes decode failed({}, {}): {}", kind, version, e);
}

/// https://github.com/kubernetes/kubernetes/blob/master/staging/src/k8s.io/api/core/v1/generated.proto
fn try_format_core_v1(kind: &str, version: &str, raw: &[u8]) -> Option<FormattedValue> {
    match kind {
        "Pod" => proto::k8s::io::api::core::v1::Pod::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        "Service" => proto::k8s::io::api::core::v1::Service::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        "Endpoints" => proto::k8s::io::api::core::v1::Endpoints::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        "ConfigMap" => proto::k8s::io::api::core::v1::ConfigMap::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        "Node" => proto::k8s::io::api::core::v1::Node::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        "Namespace" => proto::k8s::io::api::core::v1::Namespace::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        "RangeAllocation" => proto::k8s::io::api::core::v1::RangeAllocation::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        "ServiceAccount" => proto::k8s::io::api::core::v1::ServiceAccount::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        "Event" => proto::k8s::io::api::core::v1::Event::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        "Secret" => proto::k8s::io::api::core::v1::Secret::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        "PersistentVolumeClaim" => {
            proto::k8s::io::api::core::v1::PersistentVolumeClaim::decode(raw)
                .map_err(|e| decode_err_handle(kind, version, e))
                .map_or(None, |o| try_format_to_json(&o))
        }
        "PersistentVolume" => proto::k8s::io::api::core::v1::PersistentVolume::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        _ => None,
    }
}

/// https://github.com/kubernetes/kubernetes/blob/master/staging/src/k8s.io/api/apps/v1/generated.proto
fn try_format_apps_v1(kind: &str, version: &str, raw: &[u8]) -> Option<FormattedValue> {
    match kind {
        "Deployment" => proto::k8s::io::api::apps::v1::Deployment::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        "ControllerRevision" => proto::k8s::io::api::apps::v1::ControllerRevision::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        "DaemonSet" => proto::k8s::io::api::apps::v1::DaemonSet::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        "ReplicaSet" => proto::k8s::io::api::apps::v1::ReplicaSet::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        "StatefulSet" => proto::k8s::io::api::apps::v1::StatefulSet::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        _ => None,
    }
}

/// https://github.com/kubernetes/kubernetes/blob/master/staging/src/k8s.io/api/rbac/v1/generated.proto
fn try_format_rbac_v1(kind: &str, version: &str, raw: &[u8]) -> Option<FormattedValue> {
    match kind {
        "ClusterRole" => proto::k8s::io::api::rbac::v1::ClusterRole::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        "ClusterRoleBinding" => proto::k8s::io::api::rbac::v1::ClusterRoleBinding::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        "RoleBinding" => proto::k8s::io::api::rbac::v1::RoleBinding::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        "Role" => proto::k8s::io::api::rbac::v1::Role::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        _ => None,
    }
}

/// https://github.com/kubernetes/kubernetes/blob/master/staging/src/k8s.io/api/storage/v1/generated.proto
fn try_format_storage_v1(kind: &str, version: &str, raw: &[u8]) -> Option<FormattedValue> {
    match kind {
        "CSINode" => proto::k8s::io::api::storage::v1::CsiNode::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        "StorageClass" => proto::k8s::io::api::storage::v1::StorageClass::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        _ => None,
    }
}

/// https://github.com/kubernetes/kubernetes/blob/master/staging/src/k8s.io/api/discovery/v1/generated.proto
fn try_format_discovery_v1(kind: &str, version: &str, raw: &[u8]) -> Option<FormattedValue> {
    match kind {
        "EndpointSlice" => proto::k8s::io::api::discovery::v1::EndpointSlice::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        _ => None,
    }
}

/// https://github.com/kubernetes/kubernetes/blob/master/staging/src/k8s.io/api/discovery/v1beta1/generated.proto
fn try_format_discovery_v1beta1(kind: &str, version: &str, raw: &[u8]) -> Option<FormattedValue> {
    match kind {
        "EndpointSlice" => proto::k8s::io::api::discovery::v1beta1::EndpointSlice::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        _ => None,
    }
}

/// https://github.com/kubernetes/kubernetes/blob/master/staging/src/k8s.io/api/flowcontrol/v1/generated.proto
fn try_format_flowcontrol_v1(kind: &str, version: &str, raw: &[u8]) -> Option<FormattedValue> {
    match kind {
        "FlowSchema" => proto::k8s::io::api::flowcontrol::v1::FlowSchema::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        "PriorityLevelConfiguration" => {
            proto::k8s::io::api::flowcontrol::v1::PriorityLevelConfiguration::decode(raw)
                .map_err(|e| decode_err_handle(kind, version, e))
                .map_or(None, |o| try_format_to_json(&o))
        }
        _ => None,
    }
}

/// https://github.com/kubernetes/kubernetes/blob/master/staging/src/k8s.io/api/coordination/v1/generated.proto
fn try_format_coordination_v1(kind: &str, version: &str, raw: &[u8]) -> Option<FormattedValue> {
    match kind {
        "Lease" => proto::k8s::io::api::coordination::v1::Lease::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        _ => None,
    }
}

/// https://github.com/kubernetes/kubernetes/blob/master/staging/src/k8s.io/api/scheduling/v1/generated.proto
fn try_format_scheduling_v1(kind: &str, version: &str, raw: &[u8]) -> Option<FormattedValue> {
    match kind {
        "PriorityClass" => proto::k8s::io::api::scheduling::v1::PriorityClass::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        _ => None,
    }
}

/// https://github.com/kubernetes/kubernetes/blob/master/staging/src/k8s.io/api/batch/v1/generated.proto
fn try_format_batch_v1(kind: &str, version: &str, raw: &[u8]) -> Option<FormattedValue> {
    match kind {
        "CronJob" => proto::k8s::io::api::batch::v1::CronJob::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        "Job" => proto::k8s::io::api::batch::v1::Job::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        _ => None,
    }
}

/// https://github.com/kubernetes/kubernetes/blob/master/staging/src/k8s.io/api/networking/v1/generated.proto
fn try_format_networking_v1(kind: &str, version: &str, raw: &[u8]) -> Option<FormattedValue> {
    match kind {
        "IngressClass" => proto::k8s::io::api::networking::v1::IngressClass::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        "Ingress" => proto::k8s::io::api::networking::v1::Ingress::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        _ => None,
    }
}

/// https://github.com/kubernetes/kubernetes/blob/master/staging/src/k8s.io/api/admissionregistration/v1/generated.proto
fn try_format_admissionregistration_v1(kind: &str, version: &str, raw: &[u8]) -> Option<FormattedValue> {
    match kind {
        "ValidatingWebhookConfiguration" => proto::k8s::io::api::admissionregistration::v1::ValidatingWebhookConfiguration::decode(raw)
            .map_err(|e| decode_err_handle(kind, version, e))
            .map_or(None, |o| try_format_to_json(&o)),
        _ => None,
    }
}