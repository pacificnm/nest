//! Local loopback HTTP(S) listener for the OAuth2 redirect callback.
//!
//! Desktop apps have nowhere for a provider's redirect to land by default,
//! so this opens `127.0.0.1:<port>` for the duration of a single login
//! attempt, reads the one request the browser sends after the user
//! finishes at the provider, and shuts back down — the same approach tools
//! like `gh auth login` use. Deliberately not built on `nest-http-serve`
//! (axum/tower/etc.): reading one query string off one connection doesn't
//! need a general-purpose router.
//!
//! Some providers (Schwab included) require the registered redirect URI to
//! be `https://`, even for a `127.0.0.1` loopback — so when
//! [`OAuthClientConfig::use_https_callback`](crate::config::OAuthClientConfig)
//! is set, this generates a fresh self-signed certificate for `127.0.0.1`
//! per login attempt (via `rcgen`) and terminates TLS on it (via
//! `tokio-rustls`) before reading the request. The browser will show a
//! self-signed-certificate warning — expected, and safe to click through,
//! since the connection never leaves the local machine.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use rcgen::{CertificateParams, KeyPair};
use rustls::pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::ServerConfig;
use time::{Duration as TimeDuration, OffsetDateTime};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::time::timeout;
use tokio_rustls::TlsAcceptor;
use tracing::debug;

use crate::error::{OAuthError, OAuthResult};

/// The `code`/`state` pair (or `error`) extracted from the redirect.
#[derive(Debug)]
pub struct CallbackResult {
    /// The authorization code to exchange for a token.
    pub code: String,
    /// The CSRF state to verify against the original request.
    pub state: String,
}

const MAX_REQUEST_BYTES: usize = 8192;

/// Binds `127.0.0.1:<port>`, accepts exactly one connection, and parses its
/// query string. Fails if nothing connects within `wait_timeout`. Speaks
/// plain HTTP unless `use_tls` is set, in which case it terminates TLS with
/// a freshly generated self-signed certificate first.
pub async fn wait_for_callback(
    port: u16,
    wait_timeout: Duration,
    use_tls: bool,
) -> OAuthResult<CallbackResult> {
    let listener = TcpListener::bind(("127.0.0.1", port))
        .await
        .map_err(|err| {
            OAuthError::callback(format!(
                "failed to bind loopback listener on 127.0.0.1:{port}: {err}"
            ))
            .with_source(err)
        })?;
    debug!(
        port = port,
        use_tls = use_tls,
        "oauth callback listener bound"
    );

    let (stream, _) = timeout(wait_timeout, listener.accept())
        .await
        .map_err(|_| OAuthError::callback("timed out waiting for the OAuth redirect"))?
        .map_err(|err| {
            OAuthError::callback(format!("failed to accept loopback connection: {err}"))
                .with_source(err)
        })?;

    if use_tls {
        let acceptor = build_loopback_tls_acceptor()?;
        let tls_stream = acceptor.accept(stream).await.map_err(|err| {
            OAuthError::callback(format!("TLS handshake with the browser failed: {err}"))
                .with_source(err)
        })?;
        handle_connection(tls_stream).await
    } else {
        handle_connection(stream).await
    }
}

fn build_loopback_tls_acceptor() -> OAuthResult<TlsAcceptor> {
    // Other rustls-based dependencies in the workspace (reqwest's
    // rustls-tls, via hyper-rustls's "ring" feature) pull in a second
    // crypto backend alongside this crate's own "aws_lc_rs" default, so
    // rustls can't auto-select one — it needs an explicit process-wide
    // default installed once. `install_default` errors if one is already
    // installed (by us or anything else in the process), which is fine to
    // ignore: some provider is installed either way.
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    // rcgen::generate_simple_self_signed defaults to a 1975-01-01 to
    // 4096-01-01 validity period. Some TLS stacks (observed: Firefox/NSS)
    // reject that outright with a fatal BadCertificate alert rather than
    // the usual interactive self-signed-cert warning — curl/OpenSSL
    // accepted it fine, which is why this wasn't caught by curl-based
    // testing. A real, short-lived (this cert is regenerated fresh per
    // login attempt anyway) validity window avoids the issue.
    let mut params = CertificateParams::new(vec!["127.0.0.1".to_string()]).map_err(|err| {
        OAuthError::callback(format!("failed to build certificate params: {err}")).with_source(err)
    })?;
    let now = OffsetDateTime::now_utc();
    params.not_before = now - TimeDuration::days(1);
    params.not_after = now + TimeDuration::days(365);

    let key_pair = KeyPair::generate().map_err(|err| {
        OAuthError::callback(format!("failed to generate a certificate key pair: {err}"))
            .with_source(err)
    })?;
    let cert = params.self_signed(&key_pair).map_err(|err| {
        OAuthError::callback(format!("failed to generate a self-signed certificate: {err}"))
            .with_source(err)
    })?;
    let key_der = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key_pair.serialize_der()));

    let server_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert.der().clone()], key_der)
        .map_err(|err| {
            OAuthError::callback(format!("failed to build the TLS server config: {err}"))
                .with_source(err)
        })?;

    Ok(TlsAcceptor::from(Arc::new(server_config)))
}

async fn handle_connection<S>(mut stream: S) -> OAuthResult<CallbackResult>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut buf = [0u8; MAX_REQUEST_BYTES];
    let bytes_read = stream.read(&mut buf).await.map_err(|err| {
        OAuthError::callback(format!("failed to read callback request: {err}")).with_source(err)
    })?;
    let request = String::from_utf8_lossy(&buf[..bytes_read]);

    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .ok_or_else(|| OAuthError::callback("malformed callback request"))?;

    let query = path.split_once('?').map(|(_, query)| query).unwrap_or("");
    let params: HashMap<String, String> = url::form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect();

    respond(&mut stream, params.contains_key("code")).await;

    if let Some(error) = params.get("error") {
        return Err(OAuthError::access_denied(format!(
            "authorization request was denied: {error}"
        )));
    }

    let code = params
        .get("code")
        .cloned()
        .ok_or_else(|| OAuthError::callback("callback is missing the 'code' parameter"))?;
    let state = params
        .get("state")
        .cloned()
        .ok_or_else(|| OAuthError::callback("callback is missing the 'state' parameter"))?;

    Ok(CallbackResult { code, state })
}

async fn respond<S>(stream: &mut S, success: bool)
where
    S: AsyncWrite + Unpin,
{
    let body = if success {
        "<html><body>Login complete. You can close this window.</body></html>"
    } else {
        "<html><body>Login failed. You can close this window and try again.</body></html>"
    };
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    // Best-effort: the browser tab closing before this flushes shouldn't
    // fail the login, which already succeeded from the provider's side.
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.shutdown().await;
}

#[cfg(test)]
mod tests {
    use tokio::net::TcpStream;

    use super::*;

    async fn send_request(port: u16, path: &str) {
        let mut stream = TcpStream::connect(("127.0.0.1", port))
            .await
            .expect("connect to callback listener");
        stream
            .write_all(format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n").as_bytes())
            .await
            .expect("write request");
    }

    #[tokio::test]
    async fn parses_code_and_state_from_a_successful_callback() {
        let port = 51_764;
        let server = tokio::spawn(wait_for_callback(port, Duration::from_secs(5), false));
        tokio::time::sleep(Duration::from_millis(50)).await;
        send_request(
            port,
            "/callback?code=auth-code-value&state=csrf-state-value",
        )
        .await;

        let result = server.await.expect("task").expect("callback");
        assert_eq!(result.code, "auth-code-value");
        assert_eq!(result.state, "csrf-state-value");
    }

    #[tokio::test]
    async fn surfaces_provider_denial_as_access_denied() {
        let port = 51_765;
        let server = tokio::spawn(wait_for_callback(port, Duration::from_secs(5), false));
        tokio::time::sleep(Duration::from_millis(50)).await;
        send_request(port, "/callback?error=access_denied&state=csrf-state-value").await;

        let error = server.await.expect("task").expect_err("should error");
        assert_eq!(error.kind(), crate::error::OAuthErrorKind::AccessDenied);
    }

    #[tokio::test]
    async fn times_out_if_nothing_connects() {
        let port = 51_766;
        let error = wait_for_callback(port, Duration::from_millis(100), false)
            .await
            .expect_err("should time out");
        assert_eq!(error.kind(), crate::error::OAuthErrorKind::Callback);
    }

    #[tokio::test]
    async fn parses_code_and_state_from_a_successful_https_callback() {
        // The client side (below) needs a crypto provider resolved too, and
        // it can run before the spawned server task's own install call.
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

        let port = 51_767;
        let server = tokio::spawn(wait_for_callback(port, Duration::from_secs(5), true));
        tokio::time::sleep(Duration::from_millis(50)).await;

        let tcp_stream = TcpStream::connect(("127.0.0.1", port))
            .await
            .expect("connect to callback listener");

        // The client doesn't have (and can't have) the ephemeral
        // self-signed cert's issuer in a trust store, so it must be told to
        // skip verification — exactly what the flow's own browser warning
        // represents. This is a test of the TLS wiring, not of certificate
        // trust, which is why this must NOT be a pattern used outside tests.
        let config = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoCertVerification))
            .with_no_client_auth();
        let connector = tokio_rustls::TlsConnector::from(Arc::new(config));
        let server_name =
            rustls::pki_types::ServerName::IpAddress(std::net::Ipv4Addr::new(127, 0, 0, 1).into());
        let mut tls_stream = connector
            .connect(server_name, tcp_stream)
            .await
            .expect("tls handshake");
        tls_stream
            .write_all(b"GET /callback?code=auth-code-value&state=csrf-state-value HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
            .await
            .expect("write request");

        let result = server.await.expect("task").expect("callback");
        assert_eq!(result.code, "auth-code-value");
        assert_eq!(result.state, "csrf-state-value");
    }

    #[derive(Debug)]
    struct NoCertVerification;

    impl rustls::client::danger::ServerCertVerifier for NoCertVerification {
        fn verify_server_cert(
            &self,
            _end_entity: &rustls::pki_types::CertificateDer<'_>,
            _intermediates: &[rustls::pki_types::CertificateDer<'_>],
            _server_name: &rustls::pki_types::ServerName<'_>,
            _ocsp_response: &[u8],
            _now: rustls::pki_types::UnixTime,
        ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
            Ok(rustls::client::danger::ServerCertVerified::assertion())
        }

        fn verify_tls12_signature(
            &self,
            _message: &[u8],
            _cert: &rustls::pki_types::CertificateDer<'_>,
            _dss: &rustls::DigitallySignedStruct,
        ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
            Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
        }

        fn verify_tls13_signature(
            &self,
            _message: &[u8],
            _cert: &rustls::pki_types::CertificateDer<'_>,
            _dss: &rustls::DigitallySignedStruct,
        ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
            Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
        }

        fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
            rustls::crypto::CryptoProvider::get_default()
                .expect("default crypto provider installed")
                .signature_verification_algorithms
                .supported_schemes()
        }
    }
}
