//! Font Awesome font registration for egui.

use std::sync::Arc;

use egui::{Context, FontData, FontDefinitions, FontFamily};

use crate::style::IconStyle;

/// Installs Font Awesome fonts before the first frame.
///
/// Call from an eframe [`CreationContext`](eframe::CreationContext) callback.
/// Fonts become active at the start of the first egui pass.
pub fn install(ctx: &Context) {
    let mut fonts = FontDefinitions::default();
    register_fonts(&mut fonts);
    ctx.set_fonts(fonts);
}

/// Ensures Font Awesome fonts are scheduled (idempotent).
///
/// If fonts are not yet active in the current pass, rendering should wait until
/// the next pass — see [`family_available`].
pub fn ensure_installed(ctx: &Context) {
    if family_available(ctx, IconStyle::Solid) {
        return;
    }

    let mut fonts = current_definitions(ctx);
    register_fonts(&mut fonts);
    ctx.set_fonts(fonts);
    ctx.request_repaint();
}

/// Returns whether the given icon style's font family is active.
pub fn family_available(ctx: &Context, style: IconStyle) -> bool {
    let family = style.font_family();
    ctx.fonts(|fonts| fonts.families().contains(&family))
}

fn current_definitions(ctx: &Context) -> FontDefinitions {
    ctx.fonts(|fonts| fonts.lock().fonts.definitions().clone())
}

fn register_fonts(fonts: &mut FontDefinitions) {
    #[cfg(feature = "solid")]
    {
        fonts.font_data.insert(
            "fa-solid".to_owned(),
            Arc::new(FontData::from_static(include_bytes!(
                "../assets/fonts/fa-solid-900.ttf"
            ))),
        );
        fonts
            .families
            .entry(FontFamily::Name("fa-solid".into()))
            .or_default()
            .push("fa-solid".to_owned());
    }

    #[cfg(feature = "regular")]
    {
        fonts.font_data.insert(
            "fa-regular".to_owned(),
            Arc::new(FontData::from_static(include_bytes!(
                "../assets/fonts/fa-regular-400.ttf"
            ))),
        );
        fonts
            .families
            .entry(FontFamily::Name("fa-regular".into()))
            .or_default()
            .push("fa-regular".to_owned());
    }

    #[cfg(feature = "brands")]
    {
        fonts.font_data.insert(
            "fa-brands".to_owned(),
            Arc::new(FontData::from_static(include_bytes!(
                "../assets/fonts/fa-brands-400.ttf"
            ))),
        );
        fonts
            .families
            .entry(FontFamily::Name("fa-brands".into()))
            .or_default()
            .push("fa-brands".to_owned());
    }

    #[cfg(feature = "solid")]
    push_proportional_fallback(fonts, "fa-solid");
    #[cfg(feature = "regular")]
    push_proportional_fallback(fonts, "fa-regular");
    #[cfg(feature = "brands")]
    push_proportional_fallback(fonts, "fa-brands");
}

fn push_proportional_fallback(fonts: &mut FontDefinitions, font_key: &str) {
    if let Some(family) = fonts.families.get_mut(&FontFamily::Proportional) {
        if !family.iter().any(|entry| entry == font_key) {
            family.push(font_key.to_owned());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_solid_font_family() {
        let mut fonts = FontDefinitions::default();
        register_fonts(&mut fonts);
        assert!(fonts.font_data.contains_key("fa-solid"));
        assert!(
            fonts
                .families
                .get(&FontFamily::Name("fa-solid".into()))
                .is_some()
        );
    }
}
