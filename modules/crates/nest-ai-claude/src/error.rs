//! `nest_claude::ClaudeError` -> `nest_ai::AiError` mapping.

use nest_claude::ClaudeErrorKind;

/// Maps a [`nest_claude::ClaudeError`] onto [`nest_ai::AiError`].
///
/// `ClaudeErrorKind::Http`, `Auth`, `RateLimit`, `Server`, and `Api` all
/// collapse onto [`nest_ai::AiErrorKind::Request`] since `nest_ai` does not
/// distinguish transport failures from HTTP-status-coded API errors the way
/// `nest_claude` does.
pub fn claude_to_ai_error(error: nest_claude::ClaudeError) -> nest_ai::AiError {
    use ClaudeErrorKind::{Api, Auth, Config, Http, InvalidRequest, Parse, RateLimit, Server};

    let kind = match error.kind() {
        Config => nest_ai::AiErrorKind::Config,
        InvalidRequest => nest_ai::AiErrorKind::InvalidInput,
        Parse => nest_ai::AiErrorKind::Parse,
        Http | Auth | RateLimit | Server | Api => nest_ai::AiErrorKind::Request,
    };
    nest_ai::AiError::new(kind, error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_error_maps_to_config_kind() {
        let error = nest_claude::ClaudeError::config("missing key");
        let mapped = claude_to_ai_error(error);
        assert_eq!(mapped.kind(), nest_ai::AiErrorKind::Config);
    }

    #[test]
    fn parse_error_maps_to_parse_kind() {
        let error = nest_claude::ClaudeError::parse("bad json");
        let mapped = claude_to_ai_error(error);
        assert_eq!(mapped.kind(), nest_ai::AiErrorKind::Parse);
    }

    #[test]
    fn http_error_maps_to_request_kind() {
        let error = nest_claude::ClaudeError::http("connection reset");
        let mapped = claude_to_ai_error(error);
        assert_eq!(mapped.kind(), nest_ai::AiErrorKind::Request);
    }

    #[test]
    fn invalid_request_error_maps_to_invalid_input_kind() {
        let error = nest_claude::ClaudeError::new(ClaudeErrorKind::InvalidRequest, "bad request");
        let mapped = claude_to_ai_error(error);
        assert_eq!(mapped.kind(), nest_ai::AiErrorKind::InvalidInput);
    }

    #[test]
    fn auth_error_maps_to_request_kind() {
        let error = nest_claude::ClaudeError::new(ClaudeErrorKind::Auth, "unauthorized");
        let mapped = claude_to_ai_error(error);
        assert_eq!(mapped.kind(), nest_ai::AiErrorKind::Request);
    }

    #[test]
    fn rate_limit_error_maps_to_request_kind() {
        let error = nest_claude::ClaudeError::new(ClaudeErrorKind::RateLimit, "rate limited");
        let mapped = claude_to_ai_error(error);
        assert_eq!(mapped.kind(), nest_ai::AiErrorKind::Request);
    }

    #[test]
    fn server_error_maps_to_request_kind() {
        let error = nest_claude::ClaudeError::new(ClaudeErrorKind::Server, "server error");
        let mapped = claude_to_ai_error(error);
        assert_eq!(mapped.kind(), nest_ai::AiErrorKind::Request);
    }

    #[test]
    fn api_error_maps_to_request_kind() {
        let error = nest_claude::ClaudeError::new(ClaudeErrorKind::Api, "other api error");
        let mapped = claude_to_ai_error(error);
        assert_eq!(mapped.kind(), nest_ai::AiErrorKind::Request);
    }
}
