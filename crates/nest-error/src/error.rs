//! Core error type and builders.

use std::error::Error;
use std::fmt;

use crate::codes::{
    NEST_LIFECYCLE_FAILED, NEST_MODULE_CONFIG_FAILED, NEST_MODULE_DEPENDENCY_MISSING,
    NEST_SERVICE_ALREADY_REGISTERED, NEST_SERVICE_NOT_FOUND, NEST_UNKNOWN,
};
use crate::kind::NestErrorKind;
use crate::report::NestErrorReport;

/// Structured error for all Nest crates.
///
/// Uses a struct (not an enum) so every module can attach the same metadata:
/// kind, message, code, module, operation, help, and an optional source chain.
#[derive(Debug)]
pub struct NestError {
    kind: NestErrorKind,
    message: String,
    code: Option<String>,
    module: Option<String>,
    operation: Option<String>,
    help: Option<String>,
    source: Option<Box<dyn Error + Send + Sync>>,
}

/// Stable field snapshot for logging adapters (e.g. future `nest-logging`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NestErrorFields<'a> {
    /// Error category.
    pub kind: NestErrorKind,
    /// Stable error code.
    pub code: Option<&'a str>,
    /// Originating Nest module name.
    pub module: Option<&'a str>,
    /// Operation that failed.
    pub operation: Option<&'a str>,
    /// User recovery hint.
    pub help: Option<&'a str>,
    /// Primary message.
    pub message: &'a str,
}

impl NestError {
    /// Creates a new error with the given kind and message.
    pub fn new(kind: NestErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            code: None,
            module: None,
            operation: None,
            help: None,
            source: None,
        }
    }

    /// Creates a configuration error.
    pub fn config(message: impl Into<String>) -> Self {
        Self::new(NestErrorKind::Config, message)
    }

    /// Creates an I/O error.
    pub fn io(message: impl Into<String>) -> Self {
        Self::new(NestErrorKind::Io, message)
    }

    /// Creates a validation error.
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(NestErrorKind::Validation, message)
    }

    /// Creates a data error.
    pub fn data(message: impl Into<String>) -> Self {
        Self::new(NestErrorKind::Data, message)
    }

    /// Creates a command error.
    pub fn command(message: impl Into<String>) -> Self {
        Self::new(NestErrorKind::Command, message)
    }

    /// Creates a service error.
    pub fn service(message: impl Into<String>) -> Self {
        Self::new(NestErrorKind::Service, message)
    }

    /// Creates a module configuration error.
    pub fn module_error(message: impl Into<String>) -> Self {
        Self::new(NestErrorKind::Module, message).with_code(NEST_MODULE_CONFIG_FAILED)
    }

    /// Creates a plugin error.
    pub fn plugin(message: impl Into<String>) -> Self {
        Self::new(NestErrorKind::Plugin, message)
    }

    /// Creates a task error.
    pub fn task(message: impl Into<String>) -> Self {
        Self::new(NestErrorKind::Task, message)
    }

    /// Creates a UI error.
    pub fn ui(message: impl Into<String>) -> Self {
        Self::new(NestErrorKind::Ui, message)
    }

    /// Creates an auth error.
    pub fn auth(message: impl Into<String>) -> Self {
        Self::new(NestErrorKind::Auth, message)
    }

    /// Creates a network error.
    pub fn network(message: impl Into<String>) -> Self {
        Self::new(NestErrorKind::Network, message)
    }

    /// Creates an unknown error.
    pub fn unknown(message: impl Into<String>) -> Self {
        Self::new(NestErrorKind::Unknown, message).with_code(NEST_UNKNOWN)
    }

    /// Service not registered (nest-core).
    pub fn service_not_found(type_name: impl Into<String>) -> Self {
        Self::service(format!("service not registered: {}", type_name.into()))
            .with_code(NEST_SERVICE_NOT_FOUND)
    }

    /// Service already registered (nest-core).
    pub fn service_already_registered(type_name: impl Into<String>) -> Self {
        Self::service(format!("service already registered: {}", type_name.into()))
            .with_code(NEST_SERVICE_ALREADY_REGISTERED)
    }

    /// Lifecycle hook failed (nest-core).
    pub fn lifecycle(message: impl Into<String>) -> Self {
        Self::new(NestErrorKind::Module, message).with_code(NEST_LIFECYCLE_FAILED)
    }

    /// Required module dependency was not registered (nest-core).
    pub fn module_dependency_missing(
        module_id: impl Into<String>,
        dependency_id: impl Into<String>,
    ) -> Self {
        let module_id = module_id.into();
        let dependency_id = dependency_id.into();
        Self::module_error(format!(
            "module `{module_id}` requires `{dependency_id}`, which was not registered"
        ))
        .with_code(NEST_MODULE_DEPENDENCY_MISSING)
        .with_help(format!("Add `.module(...)` for `{dependency_id}` before building."))
    }

    /// Sets a stable error code.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Sets the originating Nest module name.
    pub fn with_module(mut self, module: impl Into<String>) -> Self {
        self.module = Some(module.into());
        self
    }

    /// Sets the operation that failed.
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }

    /// Sets a user-facing recovery hint.
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Attaches a source error to the chain.
    pub fn with_source(mut self, source: impl Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    /// Returns the error kind.
    pub fn kind(&self) -> NestErrorKind {
        self.kind
    }

    /// Returns the primary message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the stable error code, if set.
    pub fn code(&self) -> Option<&str> {
        self.code.as_deref()
    }

    /// Returns the originating module name, if set.
    pub fn module(&self) -> Option<&str> {
        self.module.as_deref()
    }

    /// Returns the operation that failed, if set.
    pub fn operation(&self) -> Option<&str> {
        self.operation.as_deref()
    }

    /// Returns the user recovery hint, if set.
    pub fn help(&self) -> Option<&str> {
        self.help.as_deref()
    }

    /// Returns the source error, if any.
    pub fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|s| s.as_ref() as &(dyn Error + 'static))
    }

    /// Returns a stable field snapshot for logging adapters.
    pub fn fields(&self) -> NestErrorFields<'_> {
        NestErrorFields {
            kind: self.kind,
            code: self.code(),
            module: self.module(),
            operation: self.operation(),
            help: self.help(),
            message: self.message(),
        }
    }

    /// Builds a UI/CLI-ready error report.
    pub fn report(&self) -> NestErrorReport {
        NestErrorReport::from_error(self)
    }

    /// Converts this error into a miette diagnostic report.
    #[cfg(feature = "diagnostics")]
    pub fn diagnostic_report(&self) -> miette::Report {
        miette::Report::new(self.clone())
    }
}

impl fmt::Display for NestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for NestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        NestError::source(self)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for NestError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("NestError", 6)?;
        state.serialize_field("kind", &self.kind)?;
        state.serialize_field("message", &self.message)?;
        state.serialize_field("code", &self.code)?;
        state.serialize_field("module", &self.module)?;
        state.serialize_field("operation", &self.operation)?;
        state.serialize_field("help", &self.help)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for NestError {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct NestErrorData {
            kind: NestErrorKind,
            message: String,
            code: Option<String>,
            module: Option<String>,
            operation: Option<String>,
            help: Option<String>,
        }

        let data = NestErrorData::deserialize(deserializer)?;
        Ok(Self {
            kind: data.kind,
            message: data.message,
            code: data.code,
            module: data.module,
            operation: data.operation,
            help: data.help,
            source: None,
        })
    }
}

#[cfg(feature = "diagnostics")]
impl Clone for NestError {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind,
            message: self.message.clone(),
            code: self.code.clone(),
            module: self.module.clone(),
            operation: self.operation.clone(),
            help: self.help.clone(),
            source: None,
        }
    }
}

#[cfg(feature = "diagnostics")]
impl miette::Diagnostic for NestError {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.code
            .as_ref()
            .map(|c| Box::new(c.clone()) as Box<dyn std::fmt::Display + 'a>)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.help
            .as_ref()
            .map(|h| Box::new(h.clone()) as Box<dyn std::fmt::Display + 'a>)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NEST_VALIDATION_REQUIRED: &str = "NEST_VALIDATION_REQUIRED";

    #[test]
    fn builder_chain_preserves_fields() {
        let error = NestError::validation("Email is required")
            .with_code(NEST_VALIDATION_REQUIRED)
            .with_module("nest-forms")
            .with_operation("save_customer")
            .with_help("Enter a valid email.");

        assert_eq!(error.kind(), NestErrorKind::Validation);
        assert_eq!(error.message(), "Email is required");
        assert_eq!(error.code(), Some(NEST_VALIDATION_REQUIRED));
        assert_eq!(error.module(), Some("nest-forms"));
        assert_eq!(error.operation(), Some("save_customer"));
        assert_eq!(error.help(), Some("Enter a valid email."));
    }

    #[test]
    fn fields_snapshot_for_logging() {
        let error = NestError::service_not_found("my_app::Logger");
        let fields = error.fields();

        assert_eq!(fields.kind, NestErrorKind::Service);
        assert_eq!(fields.code, Some(NEST_SERVICE_NOT_FOUND));
        assert!(fields.message.contains("Logger"));
    }

    #[test]
    fn service_constructors_set_codes() {
        let not_found = NestError::service_not_found("T");
        assert_eq!(not_found.code(), Some(NEST_SERVICE_NOT_FOUND));

        let dup = NestError::service_already_registered("T");
        assert_eq!(dup.code(), Some(NEST_SERVICE_ALREADY_REGISTERED));
    }

    #[test]
    fn display_shows_message_only() {
        let error = NestError::config("bad config");
        assert_eq!(error.to_string(), "bad config");
    }

    #[test]
    fn source_chain_preserved() {
        #[derive(Debug, thiserror::Error)]
        #[error("inner")]
        struct Inner;

        let error = NestError::io("read failed").with_source(Inner);
        assert!(error.source().is_some());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_round_trip_omits_source() {
        let error = NestError::validation("bad")
            .with_code("CODE")
            .with_module("mod");

        let json = serde_json::to_string(&error).unwrap();
        let decoded: NestError = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.kind(), NestErrorKind::Validation);
        assert_eq!(decoded.code(), Some("CODE"));
        assert!(decoded.source().is_none());
    }

    #[cfg(feature = "diagnostics")]
    #[test]
    fn diagnostic_report_builds() {
        let error = NestError::validation("bad").with_help("fix it");
        let _report = error.diagnostic_report();
    }
}
