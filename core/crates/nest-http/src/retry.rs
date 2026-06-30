//! Retry policy contracts.

use std::time::Duration;

use crate::error::{HttpError, HttpErrorKind};

/// Decides whether to retry failed HTTP requests.
pub trait RetryPolicy: Send + Sync {
    /// Returns whether another attempt should be made.
    fn should_retry(&self, attempt: u32, error: &HttpError) -> bool;

    /// Returns how long to wait before the next attempt.
    fn delay_before_retry(&self, attempt: u32) -> Duration;
}

/// Fixed maximum attempts with exponential backoff.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedRetryPolicy {
    /// Maximum number of attempts (including the first).
    pub max_attempts: u32,
    /// Base delay before the second attempt.
    pub base_delay: Duration,
}

impl Default for FixedRetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(200),
        }
    }
}

impl FixedRetryPolicy {
    /// Creates a retry policy.
    pub fn new(max_attempts: u32, base_delay: Duration) -> Self {
        Self {
            max_attempts,
            base_delay,
        }
    }
}

impl RetryPolicy for FixedRetryPolicy {
    fn should_retry(&self, attempt: u32, error: &HttpError) -> bool {
        if attempt >= self.max_attempts {
            return false;
        }
        matches!(
            error.kind(),
            HttpErrorKind::Timeout | HttpErrorKind::Connection
        ) || error
            .response_status()
            .is_some_and(|s| s.code() == 503)
    }

    fn delay_before_retry(&self, attempt: u32) -> Duration {
        let multiplier = 2u32.saturating_pow(attempt.saturating_sub(1));
        self.base_delay.saturating_mul(multiplier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codes::NEST_HTTP_TIMEOUT;
    use crate::error::HttpError;
    use crate::status::HttpStatus;

    #[test]
    fn retries_on_timeout() {
        let policy = FixedRetryPolicy::default();
        let err = HttpError::timeout("timed out");
        assert!(policy.should_retry(1, &err));
        assert!(!policy.should_retry(3, &err));
    }

    #[test]
    fn retries_on_503() {
        let policy = FixedRetryPolicy::default();
        let err = HttpError::from_status(HttpStatus::SERVICE_UNAVAILABLE, "unavailable");
        assert!(policy.should_retry(1, &err));
    }

    #[test]
    fn exponential_backoff() {
        let policy = FixedRetryPolicy::new(5, Duration::from_millis(100));
        assert_eq!(policy.delay_before_retry(1), Duration::from_millis(100));
        assert_eq!(policy.delay_before_retry(2), Duration::from_millis(200));
        assert_eq!(policy.delay_before_retry(3), Duration::from_millis(400));
    }

    #[test]
    fn timeout_has_code() {
        let err = HttpError::timeout("x");
        assert_eq!(err.code(), Some(NEST_HTTP_TIMEOUT));
    }
}
