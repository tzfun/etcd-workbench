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
pub fn try_format_value(key: &String, value: &Vec<u8>) -> Option<FormattedValue> {
    if key.starts_with("/registry/pods/") {
        try_format_pods(value)
    } else if key.starts_with("/registry/services/") {
        try_format_services(value)
    } else if key.starts_with("/registry/deployments/") {
        try_format_deployments(value)
    } else if key.starts_with("/registry/clusterroles/") {
        try_format_cluster_roles(value)
    } else if key.starts_with("/registry/clusterrolebindings/") {
        try_format_cluster_roles_bindings(value)
    } else if key.starts_with("/registry/configmaps/") {
        try_format_configmaps(value)
    } else if key.starts_with("/registry/controllerrevisions/") {
        try_format_controller_revisions(value)
    } else if key.starts_with("/registry/csinodes/") {
        try_format_csi_nodes(value)
    } else if key.starts_with("/registry/daemonsets/") {
        try_format_daemon_sets(value)
    } else if key.starts_with("/registry/endpointslices/") {
        try_format_endpoint_slices(value)
    } else {
        None
    }
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

/// parse from `k8s.io.api.core.v1#Pod`
fn try_format_pods(value: &Vec<u8>) -> Option<FormattedValue> {
    try_format(value, |kind, version, raw| {
        if kind.eq("Pod") && version.eq("v1") {
            return proto::k8s::io::api::core::v1::Pod::decode(raw)
                .map_err(|e| decode_err_handle(kind, version, e))
                .map_or(None, |o| try_format_to_json(&o));
        }
        None
    })
}

/// parse from `k8s.io.api.core.v1#Service`
/// parse from `k8s.io.api.core.v1#Endpoints`
fn try_format_services(value: &Vec<u8>) -> Option<FormattedValue> {
    try_format(value, |kind, version, raw| {
        // parse from `k8s.io.api.core.v1#Service`
        if kind.eq("Service") && version.eq("v1") {
            return proto::k8s::io::api::core::v1::Service::decode(raw)
                .map_err(|e| decode_err_handle(kind, version, e))
                .map_or(None, |o| try_format_to_json(&o));
        }

        // parse from `k8s.io.api.core.v1#Endpoints`
        if kind.eq("Endpoints") && version.eq("v1") {
            return proto::k8s::io::api::core::v1::Endpoints::decode(raw)
                .map_err(|e| decode_err_handle(kind, version, e))
                .map_or(None, |o| try_format_to_json(&o));
        }
        None
    })
}

/// parse from `k8s.io.api.apps.v1#Deployment`
fn try_format_deployments(value: &Vec<u8>) -> Option<FormattedValue> {
    try_format(value, |kind, version, raw| {
        if kind.eq("Deployment") && version.eq("apps/v1") {
            return proto::k8s::io::api::apps::v1::Deployment::decode(raw)
                .map_err(|e| decode_err_handle(kind, version, e))
                .map_or(None, |o| try_format_to_json(&o));
        }
        None
    })
}

/// parse from `k8s.io.api.rbac.v1#ClusterRole`
fn try_format_cluster_roles(value: &Vec<u8>) -> Option<FormattedValue> {
    try_format(value, |kind, version, raw| {
        if kind.eq("ClusterRole") && version.eq("rbac.authorization.k8s.io/v1") {
            return proto::k8s::io::api::rbac::v1::ClusterRole::decode(raw)
                .map_err(|e| decode_err_handle(kind, version, e))
                .map_or(None, |o| try_format_to_json(&o));
        }
        None
    })
}

fn try_format_cluster_roles_bindings(value: &Vec<u8>) -> Option<FormattedValue> {
    try_format(value, |kind, version, raw| {
        if kind.eq("ClusterRoleBinding") && version.eq("rbac.authorization.k8s.io/v1") {
            return proto::k8s::io::api::rbac::v1::ClusterRoleBinding::decode(raw)
                .map_err(|e| decode_err_handle(kind, version, e))
                .map_or(None, |o| try_format_to_json(&o));
        }
        None
    })
}

/// parse from `k8s.io.api.core.v1#ConfigMap`
fn try_format_configmaps(value: &Vec<u8>) -> Option<FormattedValue> {
    try_format(value, |kind, version, raw| {
        if kind.eq("ConfigMap") && version.eq("v1") {
            return proto::k8s::io::api::core::v1::ConfigMap::decode(raw)
                .map_err(|e| decode_err_handle(kind, version, e))
                .map_or(None, |o| try_format_to_json(&o));
        }
        None
    })
}

/// parse from `k8s.io.api.apps.v1#ControllerRevision`
fn try_format_controller_revisions(value: &Vec<u8>) -> Option<FormattedValue> {
    try_format(value, |kind, version, raw| {
        if kind.eq("ControllerRevision") && version.eq("apps/v1") {
            return proto::k8s::io::api::apps::v1::ControllerRevision::decode(raw)
                .map_err(|e| decode_err_handle(kind, version, e))
                .map_or(None, |o| try_format_to_json(&o));
        }
        None
    })
}

/// parse from `k8s.io.api.storage.v1#CSINode`
fn try_format_csi_nodes(value: &Vec<u8>) -> Option<FormattedValue> {
    try_format(value, |kind, version, raw| {
        if kind.eq("CSINode") && version.eq("storage.k8s.io/v1") {
            return proto::k8s::io::api::storage::v1::CsiNode::decode(raw)
                .map_err(|e| decode_err_handle(kind, version, e))
                .map_or(None, |o| try_format_to_json(&o));
        }
        None
    })
}
/// parse from k8s.io.api.apps.v1.DaemonSet
fn try_format_daemon_sets(value: &Vec<u8>) -> Option<FormattedValue> {
    try_format(value, |kind, version, raw| {
        if kind.eq("DaemonSet") && version.eq("apps/v1") {
            return proto::k8s::io::api::apps::v1::DaemonSet::decode(raw)
                .map_err(|e| decode_err_handle(kind, version, e))
                .map_or(None, |o| try_format_to_json(&o));
        }
        None
    })
}

fn try_format_endpoint_slices(value: &Vec<u8>) -> Option<FormattedValue> {
    try_format(value, |kind, version, raw| {
        if kind.eq("EndpointSlice") && version.eq("discovery.k8s.io/v1beta1") {
            return proto::k8s::io::api::discovery::v1beta1::EndpointSlice::decode(raw)
                .map_err(|e| decode_err_handle(kind, version, e))
                .map_or(None, |o| try_format_to_json(&o));
        }

        if kind.eq("EndpointSlice") && version.eq("discovery.k8s.io/v1") {
            return proto::k8s::io::api::discovery::v1::EndpointSlice::decode(raw)
                .map_err(|e| decode_err_handle(kind, version, e))
                .map_or(None, |o| try_format_to_json(&o));
        }
        None
    })
}
