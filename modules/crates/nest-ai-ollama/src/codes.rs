//! Stable error codes for nest-ai-ollama.

/// Generic Ollama operation failure.
pub const NEST_AI_OLLAMA_FAILED: &str = "NEST_AI_OLLAMA_FAILED";
/// Ollama HTTP or transport failure.
pub const NEST_AI_OLLAMA_REQUEST_FAILED: &str = "NEST_AI_OLLAMA_REQUEST_FAILED";
/// Ollama response parse failure.
pub const NEST_AI_OLLAMA_PARSE_FAILED: &str = "NEST_AI_OLLAMA_PARSE_FAILED";
/// Ollama configuration error.
pub const NEST_AI_OLLAMA_CONFIG: &str = "NEST_AI_OLLAMA_CONFIG";
