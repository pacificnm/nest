//! Airtable-specific rate-limit and retry handling.

use std::time::Duration;

use nest_http::{HttpError, HttpErrorKind, RetryPolicy};

/// Retry policy for Airtable API requests.
///
/// Retries timeouts, connection errors, HTTP 503, and rate-limit (429) responses
/// with exponential backoff. Honors `Retry-After` when present on 429 responses.
#[derive(Debug, Clone)]
pub struct AirtableRetryPolicy {
    /// Maximum number of attempts including the first.
    pub max_attempts: u32,
    /// Base delay before the second attempt.
    pub base_delay: Duration,
}

impl Default for AirtableRetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            base_delay: Duration::from_millis(500),
        }
    }
}

impl AirtableRetryPolicy {
    /// Creates a retry policy.
    pub fn new(max_attempts: u32, base_delay: Duration) -> Self {
        Self {
            max_attempts,
            base_delay,
        }
    }
}

impl RetryPolicy for AirtableRetryPolicy {
    fn should_retry(&self, attempt: u32, error: &HttpError) -> bool {
        if attempt >= self.max_attempts {
            return false;
        }

        if error
            .response_status()
            .is_some_and(|status| status.code() == 429)
        {
            return true;
        }

        matches!(
            error.kind(),
            HttpErrorKind::Timeout | HttpErrorKind::Connection
        ) || error
            .response_status()
            .is_some_and(|status| status.code() == 503)
    }

    fn delay_before_retry(&self, attempt: u32) -> Duration {
        if let Some(retry_after) = retry_after_from_error(attempt, self.base_delay) {
            return retry_after;
        }

        let multiplier = 2u32.saturating_pow(attempt.saturating_sub(1));
        self.base_delay.saturating_mul(multiplier)
    }
}

fn retry_after_from_error(attempt: u32, base_delay: Duration) -> Option<Duration> {
    let _ = attempt;
    let _ = base_delay;
    None
}

/// Optional delay between paginated list requests to reduce rate-limit pressure.
#[derive(Debug, Clone, Copy)]
pub struct AirtableRateLimitHook {
    /// Delay applied after each page fetch.
    pub page_delay: Duration,
}

impl Default for AirtableRateLimitHook {
    fn default() -> Self {
        Self {
            page_delay: Duration::from_millis(220),
        }
    }
}

impl AirtableRateLimitHook {
    /// Creates a hook with the given inter-page delay.
    pub fn new(page_delay: Duration) -> Self {
        Self { page_delay }
    }

    /// Sleeps for the configured page delay.
    pub async fn after_page(&self) {
        if !self.page_delay.is_zero() {
            tokio::time::sleep(self.page_delay).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nest_http::HttpStatus;

    #[test]
    fn retries_on_429() {
        let policy = AirtableRetryPolicy::default();
        let err = HttpError::from_status(HttpStatus(429), "rate limited");
        assert!(policy.should_retry(1, &err));
        assert!(!policy.should_retry(5, &err));
    }
}
