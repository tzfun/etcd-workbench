// This file is @generated by prost-build.
/// Endpoint represents a single logical "backend" implementing a service.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Endpoint {
    /// addresses of this endpoint. The contents of this field are interpreted
    /// according to the corresponding EndpointSlice addressType field. Consumers
    /// must handle different types of addresses in the context of their own
    /// capabilities. This must contain at least one address but no more than
    /// 100. These are all assumed to be fungible and clients may choose to only
    /// use the first element. Refer to: <https://issue.k8s.io/106267>
    /// +listType=set
    #[prost(string, repeated, tag = "1")]
    #[serde(skip_serializing_if = "::prost::alloc::vec::Vec::is_empty")]
    pub addresses: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// conditions contains information about the current status of the endpoint.
    #[prost(message, optional, tag = "2")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub conditions: ::core::option::Option<EndpointConditions>,
    /// hostname of this endpoint. This field may be used by consumers of
    /// endpoints to distinguish endpoints from each other (e.g. in DNS names).
    /// Multiple endpoints which use the same hostname should be considered
    /// fungible (e.g. multiple A values in DNS). Must be lowercase and pass DNS
    /// Label (RFC 1123) validation.
    /// +optional
    #[prost(string, optional, tag = "3")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub hostname: ::core::option::Option<::prost::alloc::string::String>,
    /// targetRef is a reference to a Kubernetes object that represents this
    /// endpoint.
    /// +optional
    #[prost(message, optional, tag = "4")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub target_ref: ::core::option::Option<super::super::core::v1::ObjectReference>,
    /// topology contains arbitrary topology information associated with the
    /// endpoint. These key/value pairs must conform with the label format.
    /// <https://kubernetes.io/docs/concepts/overview/working-with-objects/labels>
    /// Topology may include a maximum of 16 key/value pairs. This includes, but
    /// is not limited to the following well known keys:
    /// * kubernetes.io/hostname: the value indicates the hostname of the node
    ///    where the endpoint is located. This should match the corresponding
    ///    node label.
    /// * topology.kubernetes.io/zone: the value indicates the zone where the
    ///    endpoint is located. This should match the corresponding node label.
    /// * topology.kubernetes.io/region: the value indicates the region where the
    ///    endpoint is located. This should match the corresponding node label.
    /// This field is deprecated and will be removed in future api versions.
    /// +optional
    #[prost(map = "string, string", tag = "5")]
    pub topology: ::std::collections::HashMap<
        ::prost::alloc::string::String,
        ::prost::alloc::string::String,
    >,
    /// nodeName represents the name of the Node hosting this endpoint. This can
    /// be used to determine endpoints local to a Node.
    /// +optional
    #[prost(string, optional, tag = "6")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub node_name: ::core::option::Option<::prost::alloc::string::String>,
    /// hints contains information associated with how an endpoint should be
    /// consumed.
    /// +featureGate=TopologyAwareHints
    /// +optional
    #[prost(message, optional, tag = "7")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub hints: ::core::option::Option<EndpointHints>,
}
/// EndpointConditions represents the current condition of an endpoint.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct EndpointConditions {
    /// ready indicates that this endpoint is prepared to receive traffic,
    /// according to whatever system is managing the endpoint. A nil value
    /// indicates an unknown state. In most cases consumers should interpret this
    /// unknown state as ready. For compatibility reasons, ready should never be
    /// "true" for terminating endpoints.
    /// +optional
    #[prost(bool, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub ready: ::core::option::Option<bool>,
    /// serving is identical to ready except that it is set regardless of the
    /// terminating state of endpoints. This condition should be set to true for
    /// a ready endpoint that is terminating. If nil, consumers should defer to
    /// the ready condition.
    /// +optional
    #[prost(bool, optional, tag = "2")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub serving: ::core::option::Option<bool>,
    /// terminating indicates that this endpoint is terminating. A nil value
    /// indicates an unknown state. Consumers should interpret this unknown state
    /// to mean that the endpoint is not terminating.
    /// +optional
    #[prost(bool, optional, tag = "3")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub terminating: ::core::option::Option<bool>,
}
/// EndpointHints provides hints describing how an endpoint should be consumed.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EndpointHints {
    /// forZones indicates the zone(s) this endpoint should be consumed by to
    /// enable topology aware routing. May contain a maximum of 8 entries.
    /// +listType=atomic
    #[prost(message, repeated, tag = "1")]
    #[serde(skip_serializing_if = "::prost::alloc::vec::Vec::is_empty")]
    pub for_zones: ::prost::alloc::vec::Vec<ForZone>,
}
/// EndpointPort represents a Port used by an EndpointSlice
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EndpointPort {
    /// name represents the name of this port. All ports in an EndpointSlice must have a unique name.
    /// If the EndpointSlice is derived from a Kubernetes service, this corresponds to the Service.ports\[\].name.
    /// Name must either be an empty string or pass DNS_LABEL validation:
    /// * must be no more than 63 characters long.
    /// * must consist of lower case alphanumeric characters or '-'.
    /// * must start and end with an alphanumeric character.
    /// Default is empty string.
    #[prost(string, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    /// protocol represents the IP protocol for this port.
    /// Must be UDP, TCP, or SCTP.
    /// Default is TCP.
    #[prost(string, optional, tag = "2")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub protocol: ::core::option::Option<::prost::alloc::string::String>,
    /// port represents the port number of the endpoint.
    /// If this is not specified, ports are not restricted and must be
    /// interpreted in the context of the specific consumer.
    #[prost(int32, optional, tag = "3")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub port: ::core::option::Option<i32>,
    /// appProtocol represents the application protocol for this port.
    /// This field follows standard Kubernetes label syntax.
    /// Un-prefixed names are reserved for IANA standard service names (as per
    /// RFC-6335 and <https://www.iana.org/assignments/service-names>).
    /// Non-standard protocols should use prefixed names such as
    /// mycompany.com/my-custom-protocol.
    /// +optional
    #[prost(string, optional, tag = "4")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub app_protocol: ::core::option::Option<::prost::alloc::string::String>,
}
/// EndpointSlice represents a subset of the endpoints that implement a service.
/// For a given service there may be multiple EndpointSlice objects, selected by
/// labels, which must be joined to produce the full set of endpoints.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EndpointSlice {
    /// Standard object's metadata.
    /// +optional
    #[prost(message, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub metadata: ::core::option::Option<
        super::super::super::apimachinery::pkg::apis::meta::v1::ObjectMeta,
    >,
    /// addressType specifies the type of address carried by this EndpointSlice.
    /// All addresses in this slice must be the same type. This field is
    /// immutable after creation. The following address types are currently
    /// supported:
    /// * IPv4: Represents an IPv4 Address.
    /// * IPv6: Represents an IPv6 Address.
    /// * FQDN: Represents a Fully Qualified Domain Name.
    #[prost(string, optional, tag = "4")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub address_type: ::core::option::Option<::prost::alloc::string::String>,
    /// endpoints is a list of unique endpoints in this slice. Each slice may
    /// include a maximum of 1000 endpoints.
    /// +listType=atomic
    #[prost(message, repeated, tag = "2")]
    #[serde(skip_serializing_if = "::prost::alloc::vec::Vec::is_empty")]
    pub endpoints: ::prost::alloc::vec::Vec<Endpoint>,
    /// ports specifies the list of network ports exposed by each endpoint in
    /// this slice. Each port must have a unique name. When ports is empty, it
    /// indicates that there are no defined ports. When a port is defined with a
    /// nil port value, it indicates "all ports". Each slice may include a
    /// maximum of 100 ports.
    /// +optional
    /// +listType=atomic
    #[prost(message, repeated, tag = "3")]
    #[serde(skip_serializing_if = "::prost::alloc::vec::Vec::is_empty")]
    pub ports: ::prost::alloc::vec::Vec<EndpointPort>,
}
/// EndpointSliceList represents a list of endpoint slices
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EndpointSliceList {
    /// Standard list metadata.
    /// +optional
    #[prost(message, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub metadata: ::core::option::Option<
        super::super::super::apimachinery::pkg::apis::meta::v1::ListMeta,
    >,
    /// items is the list of endpoint slices
    #[prost(message, repeated, tag = "2")]
    #[serde(skip_serializing_if = "::prost::alloc::vec::Vec::is_empty")]
    pub items: ::prost::alloc::vec::Vec<EndpointSlice>,
}
/// ForZone provides information about which zones should consume this endpoint.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ForZone {
    /// name represents the name of the zone.
    #[prost(string, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
}
