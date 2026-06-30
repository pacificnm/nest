//! Theme adapter hook for future `nest-egui-theme` integration.

#![allow(dead_code)]

use egui::Visuals;
use nest_core::AppContext;
use nest_theme::ThemeMode;
use nest_error::NestResult;
use nest_theme::ThemeService;

/// Applies the active Nest theme to egui visuals (v1 stub).
///
/// Full token mapping lives in `nest-egui-theme` (deferred).
pub fn apply_active_theme(visuals: &mut Visuals, ctx: &AppContext) -> NestResult<()> {
    let theme_service = ctx.service::<ThemeService>()?;
    if let Ok(theme) = theme_service.active_theme() {
        match theme.mode {
            ThemeMode::Dark => *visuals = Visuals::dark(),
            ThemeMode::Light => *visuals = Visuals::light(),
        }
    }
    Ok(())
}
