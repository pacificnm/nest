//! Built-in Tauri commands bridging Nest services to the React UI.

use nest_error::NestResult;
use nest_react_theme::ReactThemeAdapter;
use nest_theme::{ThemeAdapter, ThemeService};
#[cfg(feature = "runtime")]
use tauri::State;

use crate::state::NestHostState;

/// Returns application metadata for the webview splash / about UI.
#[tauri::command]
pub fn nest_app_metadata(state: State<'_, NestHostState>) -> NestResult<AppMetadataResponse> {
    Ok(AppMetadataResponse {
        name: state.app_name.clone(),
        title: state.runtime_config.title.clone(),
    })
}

/// Returns active theme CSS variables for injection into the webview.
#[tauri::command]
pub fn nest_theme_css(state: State<'_, NestHostState>) -> NestResult<ThemeCssResponse> {
    let themes = state.context.service::<ThemeService>()?;
    let active = themes.active_theme()?;
    let css = ReactThemeAdapter::adapt(&active)?;
    let root_block = css.to_root_block();
    Ok(ThemeCssResponse {
        id: css.id,
        mode: css.mode,
        variables: css.variables,
        root_block,
    })
}

/// Fetches (or loads from cache) image bytes for the React webview.
#[cfg(feature = "images")]
#[tauri::command]
pub fn nest_image_fetch(
    state: State<'_, NestHostState>,
    request: crate::image::ImageFetchRequest,
) -> NestResult<crate::image::ImageFetchResponse> {
    crate::image::fetch_image(
        state.context.as_ref(),
        &request.url,
        request.tags.as_deref().unwrap_or(&[]),
    )
}

/// Invalidates all cached images with the given tag.
#[cfg(feature = "images")]
#[tauri::command]
pub fn nest_image_invalidate_tag(
    state: State<'_, NestHostState>,
    request: crate::image::ImageInvalidateTagRequest,
) -> NestResult<crate::image::ImageInvalidateTagResponse> {
    crate::image::invalidate_image_tag(state.context.as_ref(), &request.tag)
}

/// Registers built-in Nest IPC handlers on a Tauri builder.
///
/// `generate_handler!` must live in the same module as `#[tauri::command]` functions.
pub fn attach_invoke_handler<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    #[cfg(feature = "images")]
    {
        builder.invoke_handler(tauri::generate_handler![
            nest_app_metadata,
            nest_theme_css,
            nest_image_fetch,
            nest_image_invalidate_tag,
        ])
    }

    #[cfg(not(feature = "images"))]
    {
        builder.invoke_handler(tauri::generate_handler![nest_app_metadata, nest_theme_css])
    }
}

/// Serializable app metadata for IPC.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppMetadataResponse {
    /// Application name from [`nest_app::AppMetadata`].
    pub name: String,
    /// Window title from runtime config.
    pub title: String,
}

/// Active theme CSS payload for the React layer.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThemeCssResponse {
    /// Theme id.
    pub id: String,
    /// Light or dark mode.
    pub mode: nest_design::ThemeMode,
    /// Individual CSS custom properties.
    pub variables: std::collections::BTreeMap<String, String>,
    /// Ready-to-inject `:root { … }` block.
    pub root_block: String,
}
