use log::debug;
use prost::Message;
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

fn try_format_unknown(value: &Vec<u8>) -> Option<Unknown> {
    let prefix_len = PROTO_PREFIX.len();
    let value_len = value.len();
    if prefix_len >= value_len {
        return None;
    }
    let new_value = value[prefix_len..].to_vec();

    let result = Unknown::decode(new_value.as_slice());
    match result {
        Ok(unknown) => Some(unknown),
        Err(e) => {
            debug!("decode k8s.io.apimachinery.pkg.runtime#Unknown failed: {e}");
            None
        }
    }
}

fn try_fromat_to_json<T>(value: &T, name: &str) -> Option<FormattedValue>
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
            debug!("serde_json::to_string({name}) failed: {e}");
        }
    }
    None
}

/// parse from `k8s.io.api.core.v1#Pod`
fn try_format_pods(value: &Vec<u8>) -> Option<FormattedValue> {
    const NAME: &'static str = "Pod";
    if let Some(unknwon) = try_format_unknown(value) {
        if let Some(type_meta) = &unknwon.type_meta {
            let version = type_meta.api_version();
            let kind = type_meta.kind();

            if kind.eq(NAME) {
                if version.eq("v1") {
                    let raw = unknwon.raw();
                    let result = proto::k8s::io::api::core::v1::Pod::decode(raw);

                    match result {
                        Ok(obj) => {
                            return try_fromat_to_json(&obj, NAME);
                        }
                        Err(e) => {
                            debug!("decode k8s.io.api.core.v1.{NAME} failed: {e}");
                        }
                    }
                }
            } else {
                debug!("unknown kind: {}, expect {}.", kind, NAME);
            }
        }
    }

    None
}

/// parse from `k8s.io.api.core.v1#Service`
fn try_format_services(value: &Vec<u8>) -> Option<FormattedValue> {
    const NAME: &'static str = "Service";
    if let Some(unknwon) = try_format_unknown(value) {
        if let Some(type_meta) = &unknwon.type_meta {
            let version = type_meta.api_version();
            let kind = type_meta.kind();

            if kind.eq(NAME) {
                if version.eq("v1") {
                    let raw = unknwon.raw();
                    let result = proto::k8s::io::api::core::v1::Pod::decode(raw);

                    match result {
                        Ok(obj) => {
                            return try_fromat_to_json(&obj, NAME);
                        }
                        Err(e) => {
                            debug!("decode k8s.io.api.core.v1.{NAME} failed: {e}");
                        }
                    }
                }
            } else {
                debug!("unknown kind: {}, expect {}.", kind, NAME);
            }
        }
    }

    None
}

/// parse from `k8s.io.api.apps.v1#Deployment`
fn try_format_deployments(value: &Vec<u8>) -> Option<FormattedValue> {
    const NAME: &'static str = "Deployment";
    if let Some(unknwon) = try_format_unknown(value) {
        if let Some(type_meta) = &unknwon.type_meta {
            let version = type_meta.api_version();
            let kind = type_meta.kind();

            if kind.eq(NAME) {
                if version.eq("v1") {
                    let raw = unknwon.raw();
                    let result = proto::k8s::io::api::apps::v1::Deployment::decode(raw);

                    match result {
                        Ok(obj) => {
                            return try_fromat_to_json(&obj, NAME);
                        }
                        Err(e) => {
                            debug!("decode k8s.io.api.apps.v1.{NAME} failed: {e}");
                        }
                    }
                }
            } else {
                debug!("unknown kind: {}, expect {}.", kind, NAME);
            }
        }
    }

    None
}

/// parse from `k8s.io.api.rbac.v1#ClusterRole`
fn try_format_clusterroles(value: &Vec<u8>) -> Option<FormattedValue> {
    const NAME: &'static str = "ClusterRole";
    if let Some(unknwon) = try_format_unknown(value) {
        if let Some(type_meta) = &unknwon.type_meta {
            let version = type_meta.api_version();
            let kind = type_meta.kind();

            if kind.eq(NAME) {
                if version.eq("v1") {
                    let raw = unknwon.raw();
                    let result = proto::k8s::io::api::rbac::v1::ClusterRole::decode(raw);

                    match result {
                        Ok(obj) => {
                            return try_fromat_to_json(&obj, NAME);
                        }
                        Err(e) => {
                            debug!("decode k8s.io.api.rbac.v1.{NAME} failed: {e}");
                        }
                    }
                }
            } else {
                debug!("unknown kind: {}, expect {}.", kind, NAME);
            }
        }
    }

    None
}

/// parse from `k8s.io.api.core.v1#ConfigMap`
fn try_format_configmaps(value: &Vec<u8>) -> Option<FormattedValue> {
    const NAME: &'static str = "ConfigMap";
    if let Some(unknwon) = try_format_unknown(value) {
        if let Some(type_meta) = &unknwon.type_meta {
            let version = type_meta.api_version();
            let kind = type_meta.kind();

            if kind.eq(NAME) {
                if version.eq("v1") {
                    let raw = unknwon.raw();
                    let result = proto::k8s::io::api::core::v1::ConfigMap::decode(raw);

                    match result {
                        Ok(obj) => {
                            return try_fromat_to_json(&obj, NAME);
                        }
                        Err(e) => {
                            debug!("decode k8s.io.api.core.v1.{NAME} failed: {e}");
                        }
                    }
                }
            } else {
                debug!("unknown kind: {}, expect {}.", kind, NAME);
            }
        }
    }

    None
}

/// parse from `k8s.io.api.apps.v1#ControllerRevision`
fn try_format_controllerrevisions(value: &Vec<u8>) -> Option<FormattedValue> {
    const NAME: &'static str = "ControllerRevision";
    if let Some(unknwon) = try_format_unknown(value) {
        if let Some(type_meta) = &unknwon.type_meta {
            let version = type_meta.api_version();
            let kind = type_meta.kind();

            if kind.eq(NAME) {
                if version.eq("v1") {
                    let raw = unknwon.raw();
                    let result = proto::k8s::io::api::apps::v1::ControllerRevision::decode(raw);

                    match result {
                        Ok(obj) => {
                            return try_fromat_to_json(&obj, NAME);
                        }
                        Err(e) => {
                            debug!("decode k8s.io.api.apps.v1.{NAME} failed: {e}");
                        }
                    }
                }
            } else {
                debug!("unknown kind: {}, expect {}.", kind, NAME);
            }
        }
    }

    None
}

/// parse from `k8s.io.api.storage.v1#CSINode`
fn try_format_csinodes(value: &Vec<u8>) -> Option<FormattedValue> {
    const NAME: &'static str = "CSINode";
    if let Some(unknwon) = try_format_unknown(value) {
        if let Some(type_meta) = &unknwon.type_meta {
            let version = type_meta.api_version();
            let kind = type_meta.kind();

            if kind.eq(NAME) {
                if version.eq("v1") {
                    let raw = unknwon.raw();
                    let result = proto::k8s::io::api::storage::v1::CsiNode::decode(raw);

                    match result {
                        Ok(obj) => {
                            return try_fromat_to_json(&obj, NAME);
                        }
                        Err(e) => {
                            debug!("decode k8s.io.api.storage.v1.{NAME} failed: {e}");
                        }
                    }
                }
            } else {
                debug!("unknown kind: {}, expect {}.", kind, NAME);
            }
        }
    }

    None
}
