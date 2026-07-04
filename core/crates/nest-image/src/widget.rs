//! egui widget for cached remote images.

use egui::{Context, Id, Image, Rect, TextureHandle, TextureOptions, Ui, Vec2};
use nest_core::AppContext;
use nest_error::NestResult;

use crate::decode::decode_to_color_image;
use crate::key::cache_key_for_url;
use crate::service::ImageService;
use crate::url::resolve_url;

/// Displays an image fetched from the Loon server (or any HTTP URL) via [`ImageService`].
pub struct RemoteImage<'a> {
    server_url: &'a str,
    path: Option<&'a str>,
    size: Vec2,
    tags: Vec<String>,
    corner_radius: f32,
    placeholder: Option<char>,
}

impl<'a> RemoteImage<'a> {
    /// Creates a remote image widget.
    pub fn new(server_url: &'a str, path: Option<&'a str>, size: Vec2) -> Self {
        Self {
            server_url,
            path,
            size,
            tags: Vec::new(),
            corner_radius: 0.0,
            placeholder: None,
        }
    }

    /// Adds cache invalidation tags.
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Rounds image corners.
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Letter shown when the path is missing or fetch fails.
    pub fn placeholder(mut self, ch: char) -> Self {
        self.placeholder = Some(ch);
        self
    }

    /// Renders the image or a placeholder.
    pub fn show(self, ui: &mut Ui, ctx: &AppContext) {
        let placeholder = self.placeholder.unwrap_or('?');
        let size = self.size;
        let corner_radius = self.corner_radius;
        if let Err(error) = self.try_show(ui, ctx) {
            tracing::warn!(%error, "remote image setup failed");
            let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
            Self::draw_placeholder_at(ui, rect, corner_radius, size, placeholder);
        }
    }

    /// Paints into an existing rect (for hero backdrops and overlays).
    pub fn paint_at(self, ui: &Ui, ctx: &AppContext, rect: Rect) {
        let placeholder = self.placeholder.unwrap_or('?');
        let corner_radius = self.corner_radius;
        if let Err(error) = self.try_paint_at(ui, ctx, rect) {
            tracing::warn!(%error, "remote image paint failed");
            Self::draw_placeholder_at(ui, rect, corner_radius, rect.size(), placeholder);
        }
    }

    /// Renders the image, returning errors for diagnostics.
    pub fn try_show(self, ui: &mut Ui, ctx: &AppContext) -> NestResult<()> {
        let (rect, response) = ui.allocate_exact_size(self.size, egui::Sense::hover());
        self.try_paint_at_with_response(ui, ctx, rect, Some(response))
    }

    fn try_paint_at(self, ui: &Ui, ctx: &AppContext, rect: Rect) -> NestResult<()> {
        self.try_paint_at_with_response(ui, ctx, rect, None)
    }

    fn try_paint_at_with_response(
        self,
        ui: &Ui,
        ctx: &AppContext,
        rect: Rect,
        response: Option<egui::Response>,
    ) -> NestResult<()> {
        let placeholder = self.placeholder.unwrap_or('?');
        let Some(path) = self.path.filter(|p| !p.is_empty()) else {
            Self::draw_placeholder_at(ui, rect, self.corner_radius, rect.size(), placeholder);
            return Ok(());
        };

        if !ui.is_rect_visible(rect) {
            return Ok(());
        }

        let images = ctx.service::<ImageService>()?;
        let url = resolve_url(self.server_url, path)?;
        let key = cache_key_for_url(&url);
        let uri = format!("nest-image://{}", key.as_str());
        let tag_refs: Vec<&str> = self.tags.iter().map(String::as_str).collect();
        let tags: Vec<&str> = if tag_refs.is_empty() {
            vec!["image"]
        } else {
            tag_refs
        };

        match load_texture(ui, &images, &uri, &url, &key, &tags) {
            Ok(Some(texture)) => {
                Image::from_texture(&texture)
                    .fit_to_exact_size(rect.size())
                    .corner_radius(self.corner_radius)
                    .paint_at(ui, rect);
            }
            Ok(None) => {
                Self::draw_placeholder_at(ui, rect, self.corner_radius, rect.size(), placeholder);
            }
            Err(error) => {
                tracing::warn!(%url, %error, "remote image fetch failed");
                if let Some(response) = response {
                    response.on_hover_text(error.to_string());
                }
                Self::draw_placeholder_at(ui, rect, self.corner_radius, rect.size(), placeholder);
            }
        }

        Ok(())
    }

    fn draw_placeholder_at(
        ui: &Ui,
        rect: Rect,
        corner_radius: f32,
        size: Vec2,
        ch: char,
    ) {
        ui.painter()
            .rect_filled(rect, corner_radius, ui.visuals().faint_bg_color);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            ch.to_string(),
            egui::FontId::proportional(size.x.min(size.y) * 0.35),
            ui.visuals().weak_text_color(),
        );
    }
}

/// Clears disk cache and in-memory egui textures for a movie after metadata/artwork changes.
pub fn invalidate_movie_images(
    ctx: &AppContext,
    ui_ctx: &Context,
    slug: &str,
) -> NestResult<u64> {
    let images = ctx.service::<ImageService>()?;
    let removed = images.invalidate_movie(slug)?;
    bump_texture_generation(ui_ctx, slug);
    ui_ctx.request_repaint();
    tracing::debug!(%slug, removed, "invalidated movie image caches");
    Ok(removed)
}

fn movie_slug_from_tags<'a>(tags: &'a [&str]) -> Option<&'a str> {
    tags.iter()
        .find_map(|tag| tag.strip_prefix("movie:"))
}

fn texture_generation(ctx: &Context, slug: &str) -> u64 {
    ctx.data(|data| {
        data.get_temp::<u64>(Id::new(("nest-image-gen", slug)))
            .unwrap_or(0)
    })
}

fn bump_texture_generation(ctx: &Context, slug: &str) {
    let generation = texture_generation(ctx, slug).wrapping_add(1);
    ctx.data_mut(|data| data.insert_temp(Id::new(("nest-image-gen", slug)), generation));
}

fn load_texture(
    ui: &Ui,
    images: &ImageService,
    uri: &str,
    url: &str,
    key: &nest_cache::CacheKey,
    tags: &[&str],
) -> NestResult<Option<TextureHandle>> {
    let generation = movie_slug_from_tags(tags)
        .map(|slug| texture_generation(ui.ctx(), slug))
        .unwrap_or(0);
    let cache_id = Id::new(("nest-image-texture", uri, generation));
    if let Some(handle) = ui.ctx().data(|data| data.get_temp::<TextureHandle>(cache_id)) {
        return Ok(Some(handle));
    }

    let bytes = images.fetch_bytes(url, key, tags)?;
    let color_image = decode_to_color_image(&bytes)?;
    let handle = ui
        .ctx()
        .load_texture(uri, color_image, TextureOptions::LINEAR);
    ui.ctx()
        .data_mut(|data| data.insert_temp(cache_id, handle.clone()));
    Ok(Some(handle))
}
