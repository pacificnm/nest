//! Theme adapter hook for future `nest-cli-theme` integration.

#![allow(dead_code)]

use nest_error::NestResult;

/// Placeholder terminal theme style output.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TuiThemeStyle {
    /// Reserved for future palette mapping.
    pub name: String,
}

/// Converts a host theme into terminal styling.
///
/// Implemented by `nest-cli-theme` in a future crate.
pub trait TuiThemeAdapter {
    /// Adapts the active theme into terminal styling.
    fn adapt_active_theme() -> NestResult<TuiThemeStyle>;
}
