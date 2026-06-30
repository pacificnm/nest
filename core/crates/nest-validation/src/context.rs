//! Validation context shared across validators.

/// Shared context for a validation run.
#[derive(Debug, Clone, Default)]
pub struct ValidationContext {
    locale: Option<String>,
    key_prefix: Option<String>,
    stop_on_first_error: bool,
}

impl ValidationContext {
    /// Creates an empty validation context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the locale for localized messages (reserved for future i18n).
    pub fn with_locale(mut self, locale: impl Into<String>) -> Self {
        self.locale = Some(locale.into());
        self
    }

    /// Prefixes validation codes (e.g. `form.` + `validation.required`).
    pub fn with_key_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = Some(prefix.into());
        self
    }

    /// When true, validators may stop after the first blocking issue.
    pub fn with_stop_on_first_error(mut self, stop: bool) -> Self {
        self.stop_on_first_error = stop;
        self
    }

    /// Returns the locale, if set.
    pub fn locale(&self) -> Option<&str> {
        self.locale.as_deref()
    }

    /// Returns the code prefix, if set.
    pub fn key_prefix(&self) -> Option<&str> {
        self.key_prefix.as_deref()
    }

    /// Whether to stop after the first blocking issue.
    pub fn stop_on_first_error(&self) -> bool {
        self.stop_on_first_error
    }

    /// Applies the configured key prefix to a validation code.
    pub fn qualify_code(&self, code: &str) -> String {
        match &self.key_prefix {
            Some(prefix) => format!("{prefix}{code}"),
            None => code.to_string(),
        }
    }
}
