// This file is @generated by prost-build.
/// PriorityClass defines mapping from a priority class name to the priority
/// integer value. The value can be any valid integer.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PriorityClass {
    /// Standard object's metadata.
    /// More info: <https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#metadata>
    /// +optional
    #[prost(message, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub metadata: ::core::option::Option<
        super::super::super::apimachinery::pkg::apis::meta::v1::ObjectMeta,
    >,
    /// value represents the integer value of this priority class. This is the actual priority that pods
    /// receive when they have the name of this class in their pod spec.
    #[prost(int32, optional, tag = "2")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub value: ::core::option::Option<i32>,
    /// globalDefault specifies whether this PriorityClass should be considered as
    /// the default priority for pods that do not have any priority class.
    /// Only one PriorityClass can be marked as `globalDefault`. However, if more than
    /// one PriorityClasses exists with their `globalDefault` field set to true,
    /// the smallest value of such global default PriorityClasses will be used as the default priority.
    /// +optional
    #[prost(bool, optional, tag = "3")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub global_default: ::core::option::Option<bool>,
    /// description is an arbitrary string that usually provides guidelines on
    /// when this priority class should be used.
    /// +optional
    #[prost(string, optional, tag = "4")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub description: ::core::option::Option<::prost::alloc::string::String>,
    /// preemptionPolicy is the Policy for preempting pods with lower priority.
    /// One of Never, PreemptLowerPriority.
    /// Defaults to PreemptLowerPriority if unset.
    /// +optional
    #[prost(string, optional, tag = "5")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub preemption_policy: ::core::option::Option<::prost::alloc::string::String>,
}
/// PriorityClassList is a collection of priority classes.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PriorityClassList {
    /// Standard list metadata
    /// More info: <https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#metadata>
    /// +optional
    #[prost(message, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub metadata: ::core::option::Option<
        super::super::super::apimachinery::pkg::apis::meta::v1::ListMeta,
    >,
    /// items is the list of PriorityClasses
    #[prost(message, repeated, tag = "2")]
    #[serde(skip_serializing_if = "::prost::alloc::vec::Vec::is_empty")]
    pub items: ::prost::alloc::vec::Vec<PriorityClass>,
}
