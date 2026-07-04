//! Font Awesome icon reference.

use egui::{FontId, RichText};

use crate::icons;
use crate::style::IconStyle;

/// A Font Awesome icon glyph with style (solid, regular, or brands).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Icon {
    glyph: char,
    style: IconStyle,
}

impl Icon {
    /// Solid icon from a Font Awesome codepoint (see [`icons::solid`]).
    pub const fn solid(glyph: char) -> Self {
        Self {
            glyph,
            style: IconStyle::Solid,
        }
    }

    /// Regular icon from a Font Awesome codepoint (see [`icons::regular`]).
    pub const fn regular(glyph: char) -> Self {
        Self {
            glyph,
            style: IconStyle::Regular,
        }
    }

    /// Brand icon from a Font Awesome codepoint (see [`icons::brands`]).
    pub const fn brands(glyph: char) -> Self {
        Self {
            glyph,
            style: IconStyle::Brands,
        }
    }

    /// Icon style (font family).
    pub fn style(self) -> IconStyle {
        self.style
    }

    /// Unicode codepoint for this icon.
    pub fn glyph(self) -> char {
        self.glyph
    }

    /// Builds egui [`RichText`] for this icon at the given size in points.
    pub fn rich_text(self, size: f32) -> RichText {
        RichText::new(self.glyph.to_string()).font(FontId::new(size, self.style.font_family()))
    }

    // --- Common solid shortcuts (Font Awesome names) ---

    /// `play` (solid).
    pub const PLAY: Self = Self::solid(icons::solid::PLAY);
    /// `pause` (solid).
    pub const PAUSE: Self = Self::solid(icons::solid::PAUSE);
    /// `stop` (solid).
    pub const STOP: Self = Self::solid(icons::solid::STOP);
    /// `eye` (solid).
    pub const EYE: Self = Self::solid(icons::solid::EYE);
    /// `pen-to-square` (solid).
    pub const PEN_TO_SQUARE: Self = Self::solid(icons::solid::PEN_TO_SQUARE);
    /// `trash` (solid).
    pub const TRASH: Self = Self::solid(icons::solid::TRASH);
    /// `plus` (solid).
    pub const PLUS: Self = Self::solid(icons::solid::PLUS);
    /// `xmark` (solid).
    pub const XMARK: Self = Self::solid(icons::solid::XMARK);
    /// `check` (solid).
    pub const CHECK: Self = Self::solid(icons::solid::CHECK);
    /// `magnifying-glass` (solid).
    pub const MAGNIFYING_GLASS: Self = Self::solid(icons::solid::MAGNIFYING_GLASS);
    /// `gear` (solid).
    pub const GEAR: Self = Self::solid(icons::solid::GEAR);
    /// `house` (solid).
    pub const HOUSE: Self = Self::solid(icons::solid::HOUSE);
    /// `film` (solid).
    pub const FILM: Self = Self::solid(icons::solid::FILM);
    /// `heart` (solid).
    pub const HEART: Self = Self::solid(icons::solid::HEART);
    /// `arrow-left` (solid).
    pub const ARROW_LEFT: Self = Self::solid(icons::solid::ARROW_LEFT);
    /// `arrow-rotate-right` (solid).
    pub const ARROW_ROTATE_RIGHT: Self = Self::solid(icons::solid::ARROW_ROTATE_RIGHT);
    /// `circle-info` (solid).
    pub const INFO: Self = Self::solid(icons::solid::CIRCLE_INFO);
    /// `triangle-exclamation` (solid).
    pub const WARNING: Self = Self::solid(icons::solid::TRIANGLE_EXCLAMATION);
    /// `circle-xmark` (solid).
    pub const ERROR: Self = Self::solid(icons::solid::CIRCLE_XMARK);
    /// `paperclip` (solid).
    pub const PAPERCLIP: Self = Self::solid(icons::solid::PAPERCLIP);
    /// `paper-plane` (solid).
    pub const PAPER_PLANE: Self = Self::solid(icons::solid::PAPER_PLANE);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcuts_use_expected_codepoints() {
        assert_eq!(Icon::PLAY.glyph(), icons::solid::PLAY);
        assert_eq!(Icon::EYE.glyph(), icons::solid::EYE);
        assert_eq!(Icon::TRASH.glyph(), icons::solid::TRASH);
        assert_eq!(Icon::PAPERCLIP.glyph(), icons::solid::PAPERCLIP);
        assert_eq!(Icon::PAPER_PLANE.glyph(), icons::solid::PAPER_PLANE);
    }
}
