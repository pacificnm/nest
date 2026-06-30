//! HTTP status codes.

use std::fmt;

/// HTTP response status code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HttpStatus(pub u16);

impl HttpStatus {
    /// 200 OK.
    pub const OK: Self = Self(200);
    /// 201 Created.
    pub const CREATED: Self = Self(201);
    /// 204 No Content.
    pub const NO_CONTENT: Self = Self(204);
    /// 400 Bad Request.
    pub const BAD_REQUEST: Self = Self(400);
    /// 401 Unauthorized.
    pub const UNAUTHORIZED: Self = Self(401);
    /// 404 Not Found.
    pub const NOT_FOUND: Self = Self(404);
    /// 500 Internal Server Error.
    pub const INTERNAL_SERVER_ERROR: Self = Self(500);
    /// 503 Service Unavailable.
    pub const SERVICE_UNAVAILABLE: Self = Self(503);

    /// Returns the numeric status code.
    pub fn code(self) -> u16 {
        self.0
    }

    /// Returns whether the status is successful (2xx).
    pub fn is_success(self) -> bool {
        (200..300).contains(&self.0)
    }

    /// Returns whether the status is a client error (4xx).
    pub fn is_client_error(self) -> bool {
        (400..500).contains(&self.0)
    }

    /// Returns whether the status is a server error (5xx).
    pub fn is_server_error(self) -> bool {
        (500..600).contains(&self.0)
    }
}

impl fmt::Display for HttpStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
