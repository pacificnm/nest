//! Icon font lifecycle service.

use egui::Context;

use crate::font;

/// Registers Font Awesome fonts on first use in an egui context.
///
/// Registered via [`crate::IconModule`]. Call [`Self::ensure_installed`] from
/// views or rely on [`crate::IconButton`] which installs automatically.
pub struct IconService;

impl IconService {
    /// Creates the icon service.
    pub fn new() -> Self {
        Self
    }

    /// Installs Font Awesome fonts before the first frame (eframe startup).
    pub fn install(&self, ctx: &Context) {
        font::install(ctx);
    }

    /// Installs Font Awesome fonts into the egui context if not already done.
    pub fn ensure_installed(&self, ctx: &Context) {
        font::ensure_installed(ctx);
    }
}

impl Default for IconService {
    fn default() -> Self {
        Self::new()
    }
}
