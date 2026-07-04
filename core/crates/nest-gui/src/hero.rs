//! Backdrop hero banner — painted background with an interactive overlay.
//!
//! Reserves layout height once via [`Ui::allocate_exact_size`], paints the
//! backdrop without affecting flow, then places widgets in a child [`Ui`] so
//! content does not double-count vertical space.

use egui::{Color32, Id, Rect, Sense, Ui, UiBuilder, Vec2};

/// Full-width backdrop hero (TMDB-style detail header).
#[derive(Debug, Clone, Copy)]
pub struct BackdropHero {
    /// Total height reserved in the parent layout.
    pub height: f32,
    /// Inset from the backdrop top-left for the interactive content block.
    pub content_inset: Vec2,
    /// Height of the content overlay region.
    pub content_height: f32,
    /// Semi-transparent color painted over the backdrop.
    pub scrim: Color32,
    /// Space below the hero before the next widget.
    pub bottom_spacing: f32,
}

impl Default for BackdropHero {
    fn default() -> Self {
        Self::detail_page()
    }
}

impl BackdropHero {
    /// TMDB-style movie/show detail defaults (420px backdrop).
    pub fn detail_page() -> Self {
        Self {
            height: 420.0,
            content_inset: Vec2::new(48.0, 80.0),
            content_height: 300.0,
            scrim: Color32::from_black_alpha(170),
            bottom_spacing: 24.0,
        }
    }

    /// Sets total reserved height.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Sets inset from the backdrop top-left for overlay content.
    pub fn content_inset(mut self, inset: Vec2) -> Self {
        self.content_inset = inset;
        self
    }

    /// Sets overlay content region height.
    pub fn content_height(mut self, height: f32) -> Self {
        self.content_height = height;
        self
    }

    /// Sets scrim color painted over the backdrop.
    pub fn scrim(mut self, color: Color32) -> Self {
        self.scrim = color;
        self
    }

    /// Sets spacing below the hero block.
    pub fn bottom_spacing(mut self, spacing: f32) -> Self {
        self.bottom_spacing = spacing;
        self
    }

    /// Interactive overlay rectangle inside the backdrop.
    pub fn content_rect(&self, backdrop_rect: Rect, width: f32) -> Rect {
        Rect::from_min_size(
            backdrop_rect.min + self.content_inset,
            Vec2::new(
                (width - self.content_inset.x * 2.0).max(200.0),
                self.content_height,
            ),
        )
    }

    /// Reserves hero height, paints backdrop + scrim, then runs overlay content.
    ///
    /// `paint_backdrop` receives the full backdrop rect — use [`egui::Ui::painter`]
    /// or image widgets that paint without advancing layout (e.g. `paint_at`).
    ///
    /// `add_content` receives a child [`Ui`] clipped to the content region; use
    /// [`hero_poster_row`] for the common poster + details column layout.
    pub fn show<R>(
        &self,
        ui: &mut Ui,
        id: impl std::hash::Hash,
        paint_backdrop: impl FnOnce(&Ui, Rect),
        add_content: impl FnOnce(&mut Ui) -> R,
    ) -> R {
        let width = ui.available_width();
        let (backdrop_rect, _) =
            ui.allocate_exact_size(Vec2::new(width, self.height), Sense::hover());

        paint_backdrop(ui, backdrop_rect);
        ui.painter()
            .rect_filled(backdrop_rect, 0.0, self.scrim);

        let content_rect = self.content_rect(backdrop_rect, width);
        let mut overlay = ui.new_child(
            UiBuilder::new()
                .id_salt(Id::new(id))
                .max_rect(content_rect),
        );
        let result = add_content(&mut overlay);

        ui.add_space(self.bottom_spacing);
        result
    }
}

/// Horizontal hero row: leading visual (poster), gap, trailing details column.
pub fn hero_poster_row<R>(
    ui: &mut Ui,
    poster: impl FnOnce(&mut Ui),
    gap: f32,
    details: impl FnOnce(&mut Ui) -> R,
) -> R {
    ui.horizontal(|ui| {
        poster(ui);
        ui.add_space(gap);
        ui.vertical(details).inner
    })
    .inner
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detail_page_defaults() {
        let hero = BackdropHero::detail_page();
        assert_eq!(hero.height, 420.0);
        assert_eq!(hero.content_inset, Vec2::new(48.0, 80.0));
        assert_eq!(hero.content_height, 300.0);
        assert_eq!(hero.bottom_spacing, 24.0);
    }

    #[test]
    fn content_rect_respects_insets() {
        let hero = BackdropHero::detail_page();
        let backdrop = Rect::from_min_size(egui::pos2(0.0, 0.0), Vec2::new(800.0, 420.0));
        let content = hero.content_rect(backdrop, 800.0);
        assert_eq!(content.min.x, 48.0);
        assert_eq!(content.min.y, 80.0);
        assert_eq!(content.width(), 704.0);
        assert_eq!(content.height(), 300.0);
    }
}
