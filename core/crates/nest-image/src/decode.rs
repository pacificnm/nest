//! Decode cached image bytes into egui textures.

use egui::ColorImage;
use nest_error::{NestError, NestResult};

/// Decodes JPEG/PNG bytes into an egui [`ColorImage`].
pub fn decode_to_color_image(bytes: &[u8]) -> NestResult<ColorImage> {
    let image = image::load_from_memory(bytes).map_err(|error| {
        NestError::validation(format!("image decode failed: {error}"))
    })?;
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok(ColorImage::from_rgba_unmultiplied(
        [width as usize, height as usize],
        &rgba,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_bytes() {
        let error = decode_to_color_image(b"not-an-image").unwrap_err();
        assert!(error.to_string().contains("decode"));
    }
}
