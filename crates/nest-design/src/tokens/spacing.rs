//! Spacing design tokens.

use serde::{Deserialize, Serialize};

/// Spacing scale in logical pixels (dp-equivalent; adapters scale per host).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpacingTokens {
    /// Extra-small spacing.
    pub xs: f32,
    /// Small spacing.
    pub sm: f32,
    /// Medium spacing.
    pub md: f32,
    /// Large spacing.
    pub lg: f32,
    /// Extra-large spacing.
    pub xl: f32,
    /// Optional double-extra-large spacing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub xxl: Option<f32>,
}
