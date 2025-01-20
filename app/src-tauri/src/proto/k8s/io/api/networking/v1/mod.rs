// This file is @generated by prost-build.
/// HTTPIngressPath associates a path with a backend. Incoming urls matching the
/// path are forwarded to the backend.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HttpIngressPath {
    /// path is matched against the path of an incoming request. Currently it can
    /// contain characters disallowed from the conventional "path" part of a URL
    /// as defined by RFC 3986. Paths must begin with a '/' and must be present
    /// when using PathType with value "Exact" or "Prefix".
    /// +optional
    #[prost(string, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub path: ::core::option::Option<::prost::alloc::string::String>,
    /// pathType determines the interpretation of the path matching. PathType can
    /// be one of the following values:
    /// * Exact: Matches the URL path exactly.
    /// * Prefix: Matches based on a URL path prefix split by '/'. Matching is
    ///    done on a path element by element basis. A path element refers is the
    ///    list of labels in the path split by the '/' separator. A request is a
    ///    match for path p if every p is an element-wise prefix of p of the
    ///    request path. Note that if the last element of the path is a substring
    ///    of the last element in request path, it is not a match (e.g. /foo/bar
    ///    matches /foo/bar/baz, but does not match /foo/barbaz).
    /// * ImplementationSpecific: Interpretation of the Path matching is up to
    ///    the IngressClass. Implementations can treat this as a separate PathType
    ///    or treat it identically to Prefix or Exact path types.
    /// Implementations are required to support all path types.
    #[prost(string, optional, tag = "3")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub path_type: ::core::option::Option<::prost::alloc::string::String>,
    /// backend defines the referenced service endpoint to which the traffic
    /// will be forwarded to.
    #[prost(message, optional, tag = "2")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub backend: ::core::option::Option<IngressBackend>,
}
/// HTTPIngressRuleValue is a list of http selectors pointing to backends.
/// In the example: <http://<host>/<path>?<searchpart>> -> backend where
/// where parts of the url correspond to RFC 3986, this resource will be used
/// to match against everything after the last '/' and before the first '?'
/// or '#'.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HttpIngressRuleValue {
    /// paths is a collection of paths that map requests to backends.
    /// +listType=atomic
    #[prost(message, repeated, tag = "1")]
    #[serde(skip_serializing_if = "::prost::alloc::vec::Vec::is_empty")]
    pub paths: ::prost::alloc::vec::Vec<HttpIngressPath>,
}
/// IPBlock describes a particular CIDR (Ex. "192.168.1.0/24","2001:db8::/64") that is allowed
/// to the pods matched by a NetworkPolicySpec's podSelector. The except entry describes CIDRs
/// that should not be included within this rule.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IpBlock {
    /// cidr is a string representing the IPBlock
    /// Valid examples are "192.168.1.0/24" or "2001:db8::/64"
    #[prost(string, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub cidr: ::core::option::Option<::prost::alloc::string::String>,
    /// except is a slice of CIDRs that should not be included within an IPBlock
    /// Valid examples are "192.168.1.0/24" or "2001:db8::/64"
    /// Except values will be rejected if they are outside the cidr range
    /// +optional
    /// +listType=atomic
    #[prost(string, repeated, tag = "2")]
    #[serde(skip_serializing_if = "::prost::alloc::vec::Vec::is_empty")]
    pub except: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// Ingress is a collection of rules that allow inbound connections to reach the
/// endpoints defined by a backend. An Ingress can be configured to give services
/// externally-reachable urls, load balance traffic, terminate SSL, offer name
/// based virtual hosting etc.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Ingress {
    /// Standard object's metadata.
    /// More info: <https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#metadata>
    /// +optional
    #[prost(message, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub metadata: ::core::option::Option<
        super::super::super::apimachinery::pkg::apis::meta::v1::ObjectMeta,
    >,
    /// spec is the desired state of the Ingress.
    /// More info: <https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#spec-and-status>
    /// +optional
    #[prost(message, optional, tag = "2")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub spec: ::core::option::Option<IngressSpec>,
    /// status is the current state of the Ingress.
    /// More info: <https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#spec-and-status>
    /// +optional
    #[prost(message, optional, tag = "3")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub status: ::core::option::Option<IngressStatus>,
}
/// IngressBackend describes all endpoints for a given service and port.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IngressBackend {
    /// service references a service as a backend.
    /// This is a mutually exclusive setting with "Resource".
    /// +optional
    #[prost(message, optional, tag = "4")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub service: ::core::option::Option<IngressServiceBackend>,
    /// resource is an ObjectRef to another Kubernetes resource in the namespace
    /// of the Ingress object. If resource is specified, a service.Name and
    /// service.Port must not be specified.
    /// This is a mutually exclusive setting with "Service".
    /// +optional
    #[prost(message, optional, tag = "3")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub resource: ::core::option::Option<
        super::super::core::v1::TypedLocalObjectReference,
    >,
}
/// IngressClass represents the class of the Ingress, referenced by the Ingress
/// Spec. The `ingressclass.kubernetes.io/is-default-class` annotation can be
/// used to indicate that an IngressClass should be considered default. When a
/// single IngressClass resource has this annotation set to true, new Ingress
/// resources without a class specified will be assigned this default class.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IngressClass {
    /// Standard object's metadata.
    /// More info: <https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#metadata>
    /// +optional
    #[prost(message, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub metadata: ::core::option::Option<
        super::super::super::apimachinery::pkg::apis::meta::v1::ObjectMeta,
    >,
    /// spec is the desired state of the IngressClass.
    /// More info: <https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#spec-and-status>
    /// +optional
    #[prost(message, optional, tag = "2")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub spec: ::core::option::Option<IngressClassSpec>,
}
/// IngressClassList is a collection of IngressClasses.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IngressClassList {
    /// Standard list metadata.
    /// +optional
    #[prost(message, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub metadata: ::core::option::Option<
        super::super::super::apimachinery::pkg::apis::meta::v1::ListMeta,
    >,
    /// items is the list of IngressClasses.
    #[prost(message, repeated, tag = "2")]
    #[serde(skip_serializing_if = "::prost::alloc::vec::Vec::is_empty")]
    pub items: ::prost::alloc::vec::Vec<IngressClass>,
}
/// IngressClassParametersReference identifies an API object. This can be used
/// to specify a cluster or namespace-scoped resource.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IngressClassParametersReference {
    /// apiGroup is the group for the resource being referenced. If APIGroup is
    /// not specified, the specified Kind must be in the core API group. For any
    /// other third-party types, APIGroup is required.
    /// +optional
    #[prost(string, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub a_pi_group: ::core::option::Option<::prost::alloc::string::String>,
    /// kind is the type of resource being referenced.
    #[prost(string, optional, tag = "2")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub kind: ::core::option::Option<::prost::alloc::string::String>,
    /// name is the name of resource being referenced.
    #[prost(string, optional, tag = "3")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    /// scope represents if this refers to a cluster or namespace scoped resource.
    /// This may be set to "Cluster" (default) or "Namespace".
    /// +optional
    #[prost(string, optional, tag = "4")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub scope: ::core::option::Option<::prost::alloc::string::String>,
    /// namespace is the namespace of the resource being referenced. This field is
    /// required when scope is set to "Namespace" and must be unset when scope is set to
    /// "Cluster".
    /// +optional
    #[prost(string, optional, tag = "5")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub namespace: ::core::option::Option<::prost::alloc::string::String>,
}
/// IngressClassSpec provides information about the class of an Ingress.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IngressClassSpec {
    /// controller refers to the name of the controller that should handle this
    /// class. This allows for different "flavors" that are controlled by the
    /// same controller. For example, you may have different parameters for the
    /// same implementing controller. This should be specified as a
    /// domain-prefixed path no more than 250 characters in length, e.g.
    /// "acme.io/ingress-controller". This field is immutable.
    #[prost(string, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub controller: ::core::option::Option<::prost::alloc::string::String>,
    /// parameters is a link to a custom resource containing additional
    /// configuration for the controller. This is optional if the controller does
    /// not require extra parameters.
    /// +optional
    #[prost(message, optional, tag = "2")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub parameters: ::core::option::Option<IngressClassParametersReference>,
}
/// IngressList is a collection of Ingress.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IngressList {
    /// Standard object's metadata.
    /// More info: <https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#metadata>
    /// +optional
    #[prost(message, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub metadata: ::core::option::Option<
        super::super::super::apimachinery::pkg::apis::meta::v1::ListMeta,
    >,
    /// items is the list of Ingress.
    #[prost(message, repeated, tag = "2")]
    #[serde(skip_serializing_if = "::prost::alloc::vec::Vec::is_empty")]
    pub items: ::prost::alloc::vec::Vec<Ingress>,
}
/// IngressLoadBalancerIngress represents the status of a load-balancer ingress point.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IngressLoadBalancerIngress {
    /// ip is set for load-balancer ingress points that are IP based.
    /// +optional
    #[prost(string, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub ip: ::core::option::Option<::prost::alloc::string::String>,
    /// hostname is set for load-balancer ingress points that are DNS based.
    /// +optional
    #[prost(string, optional, tag = "2")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub hostname: ::core::option::Option<::prost::alloc::string::String>,
    /// ports provides information about the ports exposed by this LoadBalancer.
    /// +listType=atomic
    /// +optional
    #[prost(message, repeated, tag = "4")]
    #[serde(skip_serializing_if = "::prost::alloc::vec::Vec::is_empty")]
    pub ports: ::prost::alloc::vec::Vec<IngressPortStatus>,
}
/// IngressLoadBalancerStatus represents the status of a load-balancer.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IngressLoadBalancerStatus {
    /// ingress is a list containing ingress points for the load-balancer.
    /// +optional
    /// +listType=atomic
    #[prost(message, repeated, tag = "1")]
    #[serde(skip_serializing_if = "::prost::alloc::vec::Vec::is_empty")]
    pub ingress: ::prost::alloc::vec::Vec<IngressLoadBalancerIngress>,
}
/// IngressPortStatus represents the error condition of a service port
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IngressPortStatus {
    /// port is the port number of the ingress port.
    #[prost(int32, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub port: ::core::option::Option<i32>,
    /// protocol is the protocol of the ingress port.
    /// The supported values are: "TCP", "UDP", "SCTP"
    #[prost(string, optional, tag = "2")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub protocol: ::core::option::Option<::prost::alloc::string::String>,
    /// error is to record the problem with the service port
    /// The format of the error shall comply with the following rules:
    /// - built-in error values shall be specified in this file and those shall use
    ///    CamelCase names
    /// - cloud provider specific error values must have names that comply with the
    ///    format foo.example.com/CamelCase.
    /// ---
    /// The regex it matches is (dns1123SubdomainFmt/)?(qualifiedNameFmt)
    /// +optional
    /// +kubebuilder:validation:Required
    /// +kubebuilder:validation:Pattern=`^([a-z0-9](\[-a-z0-9\]*[a-z0-9])?(\.[a-z0-9](\[-a-z0-9\]*[a-z0-9])?)*/)?(([A-Za-z0-9][-A-Za-z0-9_.]*)?\[A-Za-z0-9\])$`
    /// +kubebuilder:validation:MaxLength=316
    #[prost(string, optional, tag = "3")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub error: ::core::option::Option<::prost::alloc::string::String>,
}
/// IngressRule represents the rules mapping the paths under a specified host to
/// the related backend services. Incoming requests are first evaluated for a host
/// match, then routed to the backend associated with the matching IngressRuleValue.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IngressRule {
    /// host is the fully qualified domain name of a network host, as defined by RFC 3986.
    /// Note the following deviations from the "host" part of the
    /// URI as defined in RFC 3986:
    /// 1. IPs are not allowed. Currently an IngressRuleValue can only apply to
    ///     the IP in the Spec of the parent Ingress.
    /// 2. The `:` delimiter is not respected because ports are not allowed.
    /// 	  Currently the port of an Ingress is implicitly :80 for http and
    /// 	  :443 for https.
    /// Both these may change in the future.
    /// Incoming requests are matched against the host before the
    /// IngressRuleValue. If the host is unspecified, the Ingress routes all
    /// traffic based on the specified IngressRuleValue.
    ///
    /// host can be "precise" which is a domain name without the terminating dot of
    /// a network host (e.g. "foo.bar.com") or "wildcard", which is a domain name
    /// prefixed with a single wildcard label (e.g. "*.foo.com").
    /// The wildcard character '*' must appear by itself as the first DNS label and
    /// matches only a single label. You cannot have a wildcard label by itself (e.g. Host == "*").
    /// Requests will be matched against the Host field in the following way:
    /// 1. If host is precise, the request matches this rule if the http host header is equal to Host.
    /// 2. If host is a wildcard, then the request matches this rule if the http host header
    /// is to equal to the suffix (removing the first label) of the wildcard rule.
    /// +optional
    #[prost(string, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub host: ::core::option::Option<::prost::alloc::string::String>,
    /// IngressRuleValue represents a rule to route requests for this IngressRule.
    /// If unspecified, the rule defaults to a http catch-all. Whether that sends
    /// just traffic matching the host to the default backend or all traffic to the
    /// default backend, is left to the controller fulfilling the Ingress. Http is
    /// currently the only supported IngressRuleValue.
    /// +optional
    #[prost(message, optional, tag = "2")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub ingress_rule_value: ::core::option::Option<IngressRuleValue>,
}
/// IngressRuleValue represents a rule to apply against incoming requests. If the
/// rule is satisfied, the request is routed to the specified backend. Currently
/// mixing different types of rules in a single Ingress is disallowed, so exactly
/// one of the following must be set.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IngressRuleValue {
    /// +optional
    #[prost(message, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub http: ::core::option::Option<HttpIngressRuleValue>,
}
/// IngressServiceBackend references a Kubernetes Service as a Backend.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IngressServiceBackend {
    /// name is the referenced service. The service must exist in
    /// the same namespace as the Ingress object.
    #[prost(string, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    /// port of the referenced service. A port name or port number
    /// is required for a IngressServiceBackend.
    #[prost(message, optional, tag = "2")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub port: ::core::option::Option<ServiceBackendPort>,
}
/// IngressSpec describes the Ingress the user wishes to exist.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IngressSpec {
    /// ingressClassName is the name of an IngressClass cluster resource. Ingress
    /// controller implementations use this field to know whether they should be
    /// serving this Ingress resource, by a transitive connection
    /// (controller -> IngressClass -> Ingress resource). Although the
    /// `kubernetes.io/ingress.class` annotation (simple constant name) was never
    /// formally defined, it was widely supported by Ingress controllers to create
    /// a direct binding between Ingress controller and Ingress resources. Newly
    /// created Ingress resources should prefer using the field. However, even
    /// though the annotation is officially deprecated, for backwards compatibility
    /// reasons, ingress controllers should still honor that annotation if present.
    /// +optional
    #[prost(string, optional, tag = "4")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub ingress_class_name: ::core::option::Option<::prost::alloc::string::String>,
    /// defaultBackend is the backend that should handle requests that don't
    /// match any rule. If Rules are not specified, DefaultBackend must be specified.
    /// If DefaultBackend is not set, the handling of requests that do not match any
    /// of the rules will be up to the Ingress controller.
    /// +optional
    #[prost(message, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub default_backend: ::core::option::Option<IngressBackend>,
    /// tls represents the TLS configuration. Currently the Ingress only supports a
    /// single TLS port, 443. If multiple members of this list specify different hosts,
    /// they will be multiplexed on the same port according to the hostname specified
    /// through the SNI TLS extension, if the ingress controller fulfilling the
    /// ingress supports SNI.
    /// +listType=atomic
    /// +optional
    #[prost(message, repeated, tag = "2")]
    #[serde(skip_serializing_if = "::prost::alloc::vec::Vec::is_empty")]
    pub tls: ::prost::alloc::vec::Vec<IngressTls>,
    /// rules is a list of host rules used to configure the Ingress. If unspecified,
    /// or no rule matches, all traffic is sent to the default backend.
    /// +listType=atomic
    /// +optional
    #[prost(message, repeated, tag = "3")]
    #[serde(skip_serializing_if = "::prost::alloc::vec::Vec::is_empty")]
    pub rules: ::prost::alloc::vec::Vec<IngressRule>,
}
/// IngressStatus describe the current state of the Ingress.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IngressStatus {
    /// loadBalancer contains the current status of the load-balancer.
    /// +optional
    #[prost(message, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub load_balancer: ::core::option::Option<IngressLoadBalancerStatus>,
}
/// IngressTLS describes the transport layer security associated with an ingress.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IngressTls {
    /// hosts is a list of hosts included in the TLS certificate. The values in
    /// this list must match the name/s used in the tlsSecret. Defaults to the
    /// wildcard host setting for the loadbalancer controller fulfilling this
    /// Ingress, if left unspecified.
    /// +listType=atomic
    /// +optional
    #[prost(string, repeated, tag = "1")]
    #[serde(skip_serializing_if = "::prost::alloc::vec::Vec::is_empty")]
    pub hosts: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// secretName is the name of the secret used to terminate TLS traffic on
    /// port 443. Field is left optional to allow TLS routing based on SNI
    /// hostname alone. If the SNI host in a listener conflicts with the "Host"
    /// header field used by an IngressRule, the SNI host is used for termination
    /// and value of the "Host" header is used for routing.
    /// +optional
    #[prost(string, optional, tag = "2")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub secret_name: ::core::option::Option<::prost::alloc::string::String>,
}
/// NetworkPolicy describes what network traffic is allowed for a set of Pods
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NetworkPolicy {
    /// Standard object's metadata.
    /// More info: <https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#metadata>
    /// +optional
    #[prost(message, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub metadata: ::core::option::Option<
        super::super::super::apimachinery::pkg::apis::meta::v1::ObjectMeta,
    >,
    /// spec represents the specification of the desired behavior for this NetworkPolicy.
    /// +optional
    #[prost(message, optional, tag = "2")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub spec: ::core::option::Option<NetworkPolicySpec>,
}
/// NetworkPolicyEgressRule describes a particular set of traffic that is allowed out of pods
/// matched by a NetworkPolicySpec's podSelector. The traffic must match both ports and to.
/// This type is beta-level in 1.8
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NetworkPolicyEgressRule {
    /// ports is a list of destination ports for outgoing traffic.
    /// Each item in this list is combined using a logical OR. If this field is
    /// empty or missing, this rule matches all ports (traffic not restricted by port).
    /// If this field is present and contains at least one item, then this rule allows
    /// traffic only if the traffic matches at least one port in the list.
    /// +optional
    /// +listType=atomic
    #[prost(message, repeated, tag = "1")]
    #[serde(skip_serializing_if = "::prost::alloc::vec::Vec::is_empty")]
    pub ports: ::prost::alloc::vec::Vec<NetworkPolicyPort>,
    /// to is a list of destinations for outgoing traffic of pods selected for this rule.
    /// Items in this list are combined using a logical OR operation. If this field is
    /// empty or missing, this rule matches all destinations (traffic not restricted by
    /// destination). If this field is present and contains at least one item, this rule
    /// allows traffic only if the traffic matches at least one item in the to list.
    /// +optional
    /// +listType=atomic
    #[prost(message, repeated, tag = "2")]
    #[serde(skip_serializing_if = "::prost::alloc::vec::Vec::is_empty")]
    pub to: ::prost::alloc::vec::Vec<NetworkPolicyPeer>,
}
/// NetworkPolicyIngressRule describes a particular set of traffic that is allowed to the pods
/// matched by a NetworkPolicySpec's podSelector. The traffic must match both ports and from.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NetworkPolicyIngressRule {
    /// ports is a list of ports which should be made accessible on the pods selected for
    /// this rule. Each item in this list is combined using a logical OR. If this field is
    /// empty or missing, this rule matches all ports (traffic not restricted by port).
    /// If this field is present and contains at least one item, then this rule allows
    /// traffic only if the traffic matches at least one port in the list.
    /// +optional
    /// +listType=atomic
    #[prost(message, repeated, tag = "1")]
    #[serde(skip_serializing_if = "::prost::alloc::vec::Vec::is_empty")]
    pub ports: ::prost::alloc::vec::Vec<NetworkPolicyPort>,
    /// from is a list of sources which should be able to access the pods selected for this rule.
    /// Items in this list are combined using a logical OR operation. If this field is
    /// empty or missing, this rule matches all sources (traffic not restricted by
    /// source). If this field is present and contains at least one item, this rule
    /// allows traffic only if the traffic matches at least one item in the from list.
    /// +optional
    /// +listType=atomic
    #[prost(message, repeated, tag = "2")]
    #[serde(skip_serializing_if = "::prost::alloc::vec::Vec::is_empty")]
    pub from: ::prost::alloc::vec::Vec<NetworkPolicyPeer>,
}
/// NetworkPolicyList is a list of NetworkPolicy objects.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NetworkPolicyList {
    /// Standard list metadata.
    /// More info: <https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#metadata>
    /// +optional
    #[prost(message, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub metadata: ::core::option::Option<
        super::super::super::apimachinery::pkg::apis::meta::v1::ListMeta,
    >,
    /// items is a list of schema objects.
    #[prost(message, repeated, tag = "2")]
    #[serde(skip_serializing_if = "::prost::alloc::vec::Vec::is_empty")]
    pub items: ::prost::alloc::vec::Vec<NetworkPolicy>,
}
/// NetworkPolicyPeer describes a peer to allow traffic to/from. Only certain combinations of
/// fields are allowed
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NetworkPolicyPeer {
    /// podSelector is a label selector which selects pods. This field follows standard label
    /// selector semantics; if present but empty, it selects all pods.
    ///
    /// If namespaceSelector is also set, then the NetworkPolicyPeer as a whole selects
    /// the pods matching podSelector in the Namespaces selected by NamespaceSelector.
    /// Otherwise it selects the pods matching podSelector in the policy's own namespace.
    /// +optional
    #[prost(message, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub pod_selector: ::core::option::Option<
        super::super::super::apimachinery::pkg::apis::meta::v1::LabelSelector,
    >,
    /// namespaceSelector selects namespaces using cluster-scoped labels. This field follows
    /// standard label selector semantics; if present but empty, it selects all namespaces.
    ///
    /// If podSelector is also set, then the NetworkPolicyPeer as a whole selects
    /// the pods matching podSelector in the namespaces selected by namespaceSelector.
    /// Otherwise it selects all pods in the namespaces selected by namespaceSelector.
    /// +optional
    #[prost(message, optional, tag = "2")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub namespace_selector: ::core::option::Option<
        super::super::super::apimachinery::pkg::apis::meta::v1::LabelSelector,
    >,
    /// ipBlock defines policy on a particular IPBlock. If this field is set then
    /// neither of the other fields can be.
    /// +optional
    #[prost(message, optional, tag = "3")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub ip_block: ::core::option::Option<IpBlock>,
}
/// NetworkPolicyPort describes a port to allow traffic on
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NetworkPolicyPort {
    /// protocol represents the protocol (TCP, UDP, or SCTP) which traffic must match.
    /// If not specified, this field defaults to TCP.
    /// +optional
    #[prost(string, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub protocol: ::core::option::Option<::prost::alloc::string::String>,
    /// port represents the port on the given protocol. This can either be a numerical or named
    /// port on a pod. If this field is not provided, this matches all port names and
    /// numbers.
    /// If present, only traffic on the specified protocol AND port will be matched.
    /// +optional
    #[prost(message, optional, tag = "2")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub port: ::core::option::Option<
        super::super::super::apimachinery::pkg::util::intstr::IntOrString,
    >,
    /// endPort indicates that the range of ports from port to endPort if set, inclusive,
    /// should be allowed by the policy. This field cannot be defined if the port field
    /// is not defined or if the port field is defined as a named (string) port.
    /// The endPort must be equal or greater than port.
    /// +optional
    #[prost(int32, optional, tag = "3")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub end_port: ::core::option::Option<i32>,
}
/// NetworkPolicySpec provides the specification of a NetworkPolicy
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NetworkPolicySpec {
    /// podSelector selects the pods to which this NetworkPolicy object applies.
    /// The array of ingress rules is applied to any pods selected by this field.
    /// Multiple network policies can select the same set of pods. In this case,
    /// the ingress rules for each are combined additively.
    /// This field is NOT optional and follows standard label selector semantics.
    /// An empty podSelector matches all pods in this namespace.
    #[prost(message, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub pod_selector: ::core::option::Option<
        super::super::super::apimachinery::pkg::apis::meta::v1::LabelSelector,
    >,
    /// ingress is a list of ingress rules to be applied to the selected pods.
    /// Traffic is allowed to a pod if there are no NetworkPolicies selecting the pod
    /// (and cluster policy otherwise allows the traffic), OR if the traffic source is
    /// the pod's local node, OR if the traffic matches at least one ingress rule
    /// across all of the NetworkPolicy objects whose podSelector matches the pod. If
    /// this field is empty then this NetworkPolicy does not allow any traffic (and serves
    /// solely to ensure that the pods it selects are isolated by default)
    /// +optional
    /// +listType=atomic
    #[prost(message, repeated, tag = "2")]
    #[serde(skip_serializing_if = "::prost::alloc::vec::Vec::is_empty")]
    pub ingress: ::prost::alloc::vec::Vec<NetworkPolicyIngressRule>,
    /// egress is a list of egress rules to be applied to the selected pods. Outgoing traffic
    /// is allowed if there are no NetworkPolicies selecting the pod (and cluster policy
    /// otherwise allows the traffic), OR if the traffic matches at least one egress rule
    /// across all of the NetworkPolicy objects whose podSelector matches the pod. If
    /// this field is empty then this NetworkPolicy limits all outgoing traffic (and serves
    /// solely to ensure that the pods it selects are isolated by default).
    /// This field is beta-level in 1.8
    /// +optional
    /// +listType=atomic
    #[prost(message, repeated, tag = "3")]
    #[serde(skip_serializing_if = "::prost::alloc::vec::Vec::is_empty")]
    pub egress: ::prost::alloc::vec::Vec<NetworkPolicyEgressRule>,
    /// policyTypes is a list of rule types that the NetworkPolicy relates to.
    /// Valid options are \["Ingress"\], \["Egress"\], or \["Ingress", "Egress"\].
    /// If this field is not specified, it will default based on the existence of ingress or egress rules;
    /// policies that contain an egress section are assumed to affect egress, and all policies
    /// (whether or not they contain an ingress section) are assumed to affect ingress.
    /// If you want to write an egress-only policy, you must explicitly specify policyTypes \[ "Egress" \].
    /// Likewise, if you want to write a policy that specifies that no egress is allowed,
    /// you must specify a policyTypes value that include "Egress" (since such a policy would not include
    /// an egress section and would otherwise default to just \[ "Ingress" \]).
    /// This field is beta-level in 1.8
    /// +optional
    /// +listType=atomic
    #[prost(string, repeated, tag = "4")]
    #[serde(skip_serializing_if = "::prost::alloc::vec::Vec::is_empty")]
    pub policy_types: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// ServiceBackendPort is the service port being referenced.
/// +structType=atomic
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ServiceBackendPort {
    /// name is the name of the port on the Service.
    /// This is a mutually exclusive setting with "Number".
    /// +optional
    #[prost(string, optional, tag = "1")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    /// number is the numerical port number (e.g. 80) on the Service.
    /// This is a mutually exclusive setting with "Name".
    /// +optional
    #[prost(int32, optional, tag = "2")]
    #[serde(skip_serializing_if = "::core::option::Option::is_none")]
    pub number: ::core::option::Option<i32>,
}
