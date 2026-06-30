//! Status / semantic feedback color tokens.

use serde::{Deserialize, Serialize};

use super::color::ColorToken;

/// Semantic status colors for success, warning, error, and info states.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatusTokens {
    /// Success state color.
    pub success: ColorToken,
    /// Warning state color.
    pub warning: ColorToken,
    /// Error state color.
    pub error: ColorToken,
    /// Informational state color.
    pub info: ColorToken,
}
