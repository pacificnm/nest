//! Maps between nest-http types and reqwest.

use nest_http::{
    HeaderMap, HttpError, HttpErrorKind, HttpMethod, HttpRequest, HttpResponse, HttpResult,
    HttpStatus,
};

/// Maps a reqwest error to [`HttpError`].
pub fn map_reqwest_error(error: reqwest::Error) -> HttpError {
    let kind = if error.is_timeout() {
        HttpErrorKind::Timeout
    } else if error.is_connect() {
        HttpErrorKind::Connection
    } else if error.is_decode() {
        HttpErrorKind::Decode
    } else {
        HttpErrorKind::Connection
    };

    let url = error.url().map(|u| u.to_string());
    let mut http_error = HttpError::new(kind, error.to_string()).with_source(error);
    if http_error.kind() == HttpErrorKind::Timeout {
        http_error = http_error.with_code(nest_http::codes::NEST_HTTP_TIMEOUT);
    }
    if let Some(url) = url {
        http_error = http_error.with_url(url);
    }
    http_error
}

/// Maps an HTTP method to reqwest.
pub fn map_method(method: HttpMethod) -> reqwest::Method {
    match method {
        HttpMethod::Get => reqwest::Method::GET,
        HttpMethod::Post => reqwest::Method::POST,
        HttpMethod::Put => reqwest::Method::PUT,
        HttpMethod::Patch => reqwest::Method::PATCH,
        HttpMethod::Delete => reqwest::Method::DELETE,
        HttpMethod::Head => reqwest::Method::HEAD,
        HttpMethod::Options => reqwest::Method::OPTIONS,
    }
}

/// Builds a reqwest request from a nest-http request and default headers.
pub fn build_reqwest_request(
    client: &reqwest::Client,
    mut request: HttpRequest,
    default_headers: &HeaderMap,
) -> HttpResult<reqwest::RequestBuilder> {
    for (name, value) in default_headers.iter() {
        if request.headers.get(name).is_none() {
            request.headers.insert(name, value.to_string());
        }
    }

    let mut builder = client
        .request(map_method(request.method), &request.url)
        .headers(map_headers(&request.headers));

    if let Some(body) = request.body {
        builder = builder.body(body);
    }

    Ok(builder)
}

fn map_headers(headers: &HeaderMap) -> reqwest::header::HeaderMap {
    let mut map = reqwest::header::HeaderMap::new();
    for (name, value) in headers.iter() {
        if let (Ok(name), Ok(value)) = (
            reqwest::header::HeaderName::from_bytes(name.as_bytes()),
            reqwest::header::HeaderValue::from_str(value),
        ) {
            map.insert(name, value);
        }
    }
    map
}

/// Maps a reqwest response to nest-http.
pub async fn map_reqwest_response(response: reqwest::Response) -> HttpResult<HttpResponse> {
    let status = HttpStatus(response.status().as_u16());
    let mut headers = HeaderMap::new();
    for (name, value) in response.headers().iter() {
        if let Ok(text) = value.to_str() {
            headers.insert(name.as_str(), text);
        }
    }
    let body = response.bytes().await.map_err(map_reqwest_error)?.to_vec();
    Ok(HttpResponse {
        status,
        headers,
        body,
    })
}

/// Checks status and returns an error for non-success responses.
pub fn ensure_success(response: &HttpResponse, url: &str) -> HttpResult<()> {
    if response.status.is_success() {
        Ok(())
    } else {
        let message = api_error_message(response);
        Err(HttpError::from_status(response.status, message).with_url(url))
    }
}

fn api_error_message(response: &HttpResponse) -> String {
    if let Some(detail) = json_error_field(&response.body) {
        return detail;
    }

    let body = String::from_utf8_lossy(&response.body);
    let trimmed = body.trim();
    if trimmed.is_empty() {
        format!("HTTP {}", response.status.code())
    } else if trimmed.len() > 240 {
        format!("HTTP {}: {}…", response.status.code(), &trimmed[..240])
    } else {
        format!("HTTP {}: {}", response.status.code(), trimmed)
    }
}

fn json_error_field(body: &[u8]) -> Option<String> {
    let value: serde_json::Value = serde_json::from_slice(body).ok()?;
    value
        .get("error")
        .and_then(|error| error.as_str())
        .map(str::to_string)
}
