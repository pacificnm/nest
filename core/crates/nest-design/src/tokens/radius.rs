//! Border radius design tokens.

use serde::{Deserialize, Serialize};

/// Border radius scale in logical pixels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RadiusTokens {
    /// Small corner radius.
    pub sm: f32,
    /// Medium corner radius.
    pub md: f32,
    /// Large corner radius.
    pub lg: f32,
    /// Optional pill / full radius.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full: Option<f32>,
}
