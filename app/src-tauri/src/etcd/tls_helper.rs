//! TLS 辅助工具，仅用于 `etcd-client-tls` (rustls) feature 路径下的“跳过证书校验”能力。
//!
//! tonic 的 `ClientTlsConfig` 不直接暴露 `dangerous().with_custom_certificate_verifier`，
//! 因此当用户开启 “Insecure Skip TLS Verify” 时，我们采用如下策略：
//!
//! 1. 用 rustls + 自定义“接受任意证书”的 verifier 与服务端先做一次 TLS 握手；
//! 2. 抓取服务端 leaf certificate（DER）；
//! 3. 解析其 SubjectAltName / CommonName，得到一个用于 `domain_name` 的合法名字；
//! 4. 将该证书以 PEM 形式作为 CA 证书塞回 `TlsOptions`，并设置匹配的 `domain_name`。
//!
//! 这样既绕过了原本因 UnknownIssuer 导致的握手失败，又满足 rustls 强制的 SAN 校验。
#![cfg(feature = "etcd-client-tls")]
#![allow(dead_code)]

use std::sync::Arc;
use std::time::Duration;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use log::{debug, warn};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, Error as RustlsError, SignatureScheme};
use rustls_pki_types::TrustAnchor;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_rustls::TlsConnector;
use x509_parser::extensions::{GeneralName, ParsedExtension};
use x509_parser::prelude::FromDer;

#[derive(Debug)]
struct AcceptAnyVerifier;

impl ServerCertVerifier for AcceptAnyVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, RustlsError> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, RustlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, RustlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::ECDSA_NISTP521_SHA512,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::ED25519,
        ]
    }
}

/// 预取得到的服务端证书信息。
pub struct FetchedServerCert {
    /// PEM 编码的 leaf certificate，可作为自身 CA 加入 TlsOptions。
    pub leaf_pem: Vec<u8>,
    /// 完整证书链（DER），index 0 = leaf，最后一个 = 最顶层
    pub chain_ders: Vec<Vec<u8>>,
    /// 推导出的、可用于 rustls `domain_name` 的名字。
    /// 优先来自 SAN 的 DNS / IP，其次取 CN，最后回退到原始 host。
    pub server_name: String,
}

/// 安装一次默认 rustls CryptoProvider（多次调用是安全的）。
fn ensure_crypto_provider() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        // 忽略 install_default 失败：可能在其它地方已经安装过。
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

/// 从 PEM 格式的证书提取 TrustAnchor，用于 ClientTlsConfig::trust_anchor()
pub fn cert_pem_to_trust_anchor(
    pem: &[u8],
) -> Result<TrustAnchor<'static>, Box<dyn std::error::Error>> {
    use tokio_rustls::rustls::pki_types::pem::PemObject as _;

    // PEM → DER
    let cert_der = CertificateDer::from_pem_slice(pem)?;

    // DER → TrustAnchor（把这张证书本身当作根 CA）
    let anchor = webpki::anchor_from_trusted_cert(&cert_der)
        .map_err(|e| format!("anchor_from_trusted_cert failed: {e:?}"))?
        .to_owned(); // 转成 'static 生命周期

    Ok(anchor)
}

/// 与 `host:port` 进行一次 TLS 握手（不校验证书），抓取服务端 leaf 证书。
pub async fn fetch_server_certificate(
    host: &str,
    port: u16,
    user_domain: Option<&str>,
    handshake_timeout: Duration,
) -> Result<FetchedServerCert, String> {
    ensure_crypto_provider();

    let sni_name = user_domain
        .filter(|s| !s.is_empty())
        .unwrap_or(host)
        .to_string();
    let server_name_owned: ServerName<'static> = ServerName::try_from(sni_name.clone())
        .map_err(|e| format!("invalid server name '{}': {e}", sni_name))?
        .to_owned();

    let mut config = ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(AcceptAnyVerifier))
        .with_no_client_auth();
    // gRPC over h2，但抓取证书阶段不强制 ALPN。
    config.alpn_protocols.clear();

    let connector = TlsConnector::from(Arc::new(config));

    let tcp = timeout(handshake_timeout, TcpStream::connect((host, port)))
        .await
        .map_err(|_| format!("TCP connect to {host}:{port} timed out"))?
        .map_err(|e| format!("TCP connect to {host}:{port} failed: {e}"))?;

    let tls_stream = timeout(handshake_timeout, connector.connect(server_name_owned, tcp))
        .await
        .map_err(|_| "TLS handshake (insecure pre-fetch) timed out".to_string())?
        .map_err(|e| format!("TLS handshake (insecure pre-fetch) failed: {e}"))?;

    let (_, conn) = tls_stream.get_ref();

    // 原来只取 first()，改为取全部
    let chain = conn
        .peer_certificates()
        .ok_or_else(|| "Server did not present any certificate".to_string())?;

    if chain.is_empty() {
        return Err("Server certificate chain is empty".to_string());
    }

    let leaf_pem = der_to_pem(chain[0].as_ref());
    let server_name = pick_server_name(chain[0].as_ref(), &sni_name);

    // 保留完整链的 DER 原始字节
    let chain_ders: Vec<Vec<u8>> = chain.iter().map(|c| c.as_ref().to_vec()).collect();

    debug!(
        "Insecure TLS pre-fetch ok, host={}, picked server_name={}",
        host, server_name
    );

    Ok(FetchedServerCert {
        leaf_pem,
        chain_ders,
        server_name,
    })
}

/// 把 DER 编码的证书转换成 PEM。
fn der_to_pem(der: &[u8]) -> Vec<u8> {
    let b64 = STANDARD.encode(der);
    let mut out = String::with_capacity(b64.len() + 64);
    out.push_str("-----BEGIN CERTIFICATE-----\n");
    for chunk in b64.as_bytes().chunks(64) {
        out.push_str(std::str::from_utf8(chunk).unwrap());
        out.push('\n');
    }
    out.push_str("-----END CERTIFICATE-----\n");
    out.into_bytes()
}

/// 解析证书的 SAN / CN，挑选一个 rustls `ServerName` 可接受的名字；
/// 若都没有，则回退到调用方传入的 fallback。
fn pick_server_name(der: &[u8], fallback: &str) -> String {
    match x509_parser::certificate::X509Certificate::from_der(der) {
        Ok((_, cert)) => {
            // 优先 SAN
            for ext in cert.extensions() {
                if let ParsedExtension::SubjectAlternativeName(san) = ext.parsed_extension() {
                    for name in &san.general_names {
                        match name {
                            GeneralName::DNSName(dns) => {
                                let candidate = (*dns).to_string();
                                if ServerName::try_from(candidate.clone()).is_ok() {
                                    return candidate;
                                }
                            }
                            GeneralName::IPAddress(bytes) => {
                                if let Some(ip) = ip_bytes_to_string(bytes) {
                                    if ServerName::try_from(ip.clone()).is_ok() {
                                        return ip;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            // 退而求其次：subject 中的 CN
            if let Some(cn) = cert
                .subject()
                .iter_common_name()
                .next()
                .and_then(|cn| cn.as_str().ok())
            {
                let candidate = cn.to_string();
                if ServerName::try_from(candidate.clone()).is_ok() {
                    return candidate;
                }
            }
        }
        Err(e) => {
            warn!("Failed to parse server certificate for SAN extraction: {e}");
        }
    }

    // 最终回退到调用方提供的 host / domain。注意：rustls 仍可能因为 SAN 不匹配而拒绝。
    fallback.to_string()
}

fn ip_bytes_to_string(bytes: &[u8]) -> Option<String> {
    match bytes.len() {
        4 => Some(format!(
            "{}.{}.{}.{}",
            bytes[0], bytes[1], bytes[2], bytes[3]
        )),
        16 => {
            let mut groups = [0u16; 8];
            for i in 0..8 {
                groups[i] = ((bytes[2 * i] as u16) << 8) | bytes[2 * i + 1] as u16;
            }
            Some(std::net::Ipv6Addr::from(groups).to_string())
        }
        _ => None,
    }
}
