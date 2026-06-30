//! Stable validation error codes.

/// One or more validation errors blocked the operation.
pub const NEST_VALIDATION_FAILED: &str = "NEST_VALIDATION_FAILED";

/// Validator name was already registered.
pub const NEST_VALIDATOR_ALREADY_REGISTERED: &str = "NEST_VALIDATOR_ALREADY_REGISTERED";

/// Validator name was not found in the registry.
pub const NEST_VALIDATOR_NOT_FOUND: &str = "NEST_VALIDATOR_NOT_FOUND";
