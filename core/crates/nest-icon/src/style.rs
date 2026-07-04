//! Font Awesome icon style (font family).

use egui::FontFamily;

/// Font Awesome icon weight / family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IconStyle {
    /// Solid icons (`fa-solid-900.ttf`).
    #[default]
    Solid,
    /// Regular (outline) icons (`fa-regular-400.ttf`).
    Regular,
    /// Brand logos (`fa-brands-400.ttf`).
    Brands,
}

impl IconStyle {
    /// egui [`FontFamily`] name registered by [`crate::font::ensure_installed`].
    pub fn font_family(self) -> FontFamily {
        FontFamily::Name(match self {
            Self::Solid => "fa-solid".into(),
            Self::Regular => "fa-regular".into(),
            Self::Brands => "fa-brands".into(),
        })
    }
}
