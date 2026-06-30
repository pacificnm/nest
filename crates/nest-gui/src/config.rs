//! `[gui]` configuration section and merge logic.

use nest_config::ConfigDocument;
use nest_error::NestResult;
use serde::Deserialize;

use crate::startup::GuiStartupOptions;

/// Runtime window settings after merging defaults, config file, and CLI flags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuiRuntimeConfig {
    /// Window title.
    pub title: String,
    /// Window width in pixels.
    pub width: u32,
    /// Window height in pixels.
    pub height: u32,
    /// Enable vsync when supported.
    pub vsync: bool,
    /// Disable ANSI color in host error output.
    pub no_color: bool,
}

impl GuiRuntimeConfig {
    /// Creates runtime config with the given application name as the default title.
    pub fn with_app_name(app_name: &str) -> Self {
        Self {
            title: app_name.to_string(),
            width: 1280,
            height: 800,
            vsync: true,
            no_color: false,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct GuiSection {
    title: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    vsync: Option<bool>,
}

/// Merges GUI runtime settings: defaults < config file < CLI flags.
pub fn merge_runtime_config(
    app_name: &str,
    document: &ConfigDocument,
    startup: &GuiStartupOptions,
) -> NestResult<GuiRuntimeConfig> {
    let mut config = GuiRuntimeConfig::with_app_name(app_name);

    if document.has_section("gui") {
        let section: GuiSection = document.section("gui")?;
        if let Some(title) = section.title {
            config.title = title;
        }
        if let Some(width) = section.width {
            config.width = width;
        }
        if let Some(height) = section.height {
            config.height = height;
        }
        if let Some(vsync) = section.vsync {
            config.vsync = vsync;
        }
    }

    if let Some(title) = &startup.title {
        config.title = title.clone();
    }
    if let Some(width) = startup.width {
        config.width = width;
    }
    if let Some(height) = startup.height {
        config.height = height;
    }
    if startup.no_color {
        config.no_color = true;
    }

    Ok(config)
}
