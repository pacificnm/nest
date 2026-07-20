//! Local loopback HTTP listener for the OAuth2 redirect callback.
//!
//! Desktop apps have nowhere for a provider's redirect to land by default,
//! so this opens `http://127.0.0.1:<port>/callback` for the duration of a
//! single login attempt, reads the one request the browser sends after the
//! user finishes at the provider, and shuts back down — the same approach
//! tools like `gh auth login` use. Deliberately not built on `nest-http-serve`
//! (axum/tower/etc.): reading one query string off one connection doesn't
//! need a general-purpose router.

use std::collections::HashMap;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::time::timeout;
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
/// query string. Fails if nothing connects within `wait_timeout`.
pub async fn wait_for_callback(port: u16, wait_timeout: Duration) -> OAuthResult<CallbackResult> {
    let listener = TcpListener::bind(("127.0.0.1", port))
        .await
        .map_err(|err| {
            OAuthError::callback(format!(
                "failed to bind loopback listener on 127.0.0.1:{port}: {err}"
            ))
            .with_source(err)
        })?;
    debug!(port = port, "oauth callback listener bound");

    let (mut stream, _) = timeout(wait_timeout, listener.accept())
        .await
        .map_err(|_| OAuthError::callback("timed out waiting for the OAuth redirect"))?
        .map_err(|err| {
            OAuthError::callback(format!("failed to accept loopback connection: {err}"))
                .with_source(err)
        })?;

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

async fn respond(stream: &mut tokio::net::TcpStream, success: bool) {
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
    use super::*;

    async fn send_request(port: u16, path: &str) {
        use tokio::net::TcpStream;
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
        let server = tokio::spawn(wait_for_callback(port, Duration::from_secs(5)));
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
        let server = tokio::spawn(wait_for_callback(port, Duration::from_secs(5)));
        tokio::time::sleep(Duration::from_millis(50)).await;
        send_request(port, "/callback?error=access_denied&state=csrf-state-value").await;

        let error = server.await.expect("task").expect_err("should error");
        assert_eq!(error.kind(), crate::error::OAuthErrorKind::AccessDenied);
    }

    #[tokio::test]
    async fn times_out_if_nothing_connects() {
        let port = 51_766;
        let error = wait_for_callback(port, Duration::from_millis(100))
            .await
            .expect_err("should time out");
        assert_eq!(error.kind(), crate::error::OAuthErrorKind::Callback);
    }
}
