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

/// Lists every registered theme (id + mode) for a theme-picker UI.
#[tauri::command]
pub fn nest_theme_list(state: State<'_, NestHostState>) -> NestResult<Vec<ThemeSummary>> {
    let themes = state.context.service::<ThemeService>()?;
    let mut summaries = themes
        .list_themes()
        .into_iter()
        .map(|id| {
            let theme = themes.theme(&id)?;
            Ok(ThemeSummary {
                id: id.as_str().to_string(),
                mode: theme.mode,
            })
        })
        .collect::<NestResult<Vec<_>>>()?;
    summaries.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(summaries)
}

/// Switches the active theme and returns its CSS for immediate re-injection.
#[tauri::command]
pub fn nest_theme_set_active(
    state: State<'_, NestHostState>,
    request: ThemeSetActiveRequest,
) -> NestResult<ThemeCssResponse> {
    let themes = state.context.service::<ThemeService>()?;
    themes.set_active_theme(&nest_design::ThemeId::from(request.id))?;
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
            nest_theme_list,
            nest_theme_set_active,
            nest_image_fetch,
            nest_image_invalidate_tag,
        ])
    }

    #[cfg(not(feature = "images"))]
    {
        builder.invoke_handler(tauri::generate_handler![
            nest_app_metadata,
            nest_theme_css,
            nest_theme_list,
            nest_theme_set_active,
        ])
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

/// One entry in a theme-picker list.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThemeSummary {
    /// Theme id.
    pub id: String,
    /// Light or dark mode.
    pub mode: nest_design::ThemeMode,
}

/// Request body for [`nest_theme_set_active`].
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThemeSetActiveRequest {
    /// Id of the theme to activate.
    pub id: String,
}
