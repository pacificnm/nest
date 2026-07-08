//! Optional Nest module that registers [`crate::ThemeService`].

use nest_core::{AppBuilder, Module, ModuleId, NestResult};
use nest_design::ThemeId;

use crate::service::ThemeService;

/// Module id for [`ThemeModule`].
pub const THEME_MODULE_ID: ModuleId = ModuleId("nest-theme");

/// Registers [`ThemeService`] with optional built-in light/dark themes.
pub struct ThemeModule {
    include_defaults: bool,
    active: Option<ThemeId>,
}

impl ThemeModule {
    /// Creates a module that registers built-in themes by default.
    pub fn new() -> Self {
        Self {
            include_defaults: true,
            active: None,
        }
    }

    /// Creates a module without pre-registering built-in themes.
    pub fn without_defaults() -> Self {
        Self {
            include_defaults: false,
            active: None,
        }
    }

    /// Sets whether built-in Nest light/dark themes are registered.
    pub fn with_defaults(mut self, include_defaults: bool) -> Self {
        self.include_defaults = include_defaults;
        self
    }

    /// Overrides the initially active theme (must be a registered id).
    ///
    /// Defaults to `cbre-light` when built-in themes are included. Apps that
    /// prefer a different baseline (e.g. Kiwi's `cursor-dark`) set it here.
    pub fn with_active(mut self, id: impl Into<ThemeId>) -> Self {
        self.active = Some(id.into());
        self
    }
}

impl Default for ThemeModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for ThemeModule {
    fn id(&self) -> ModuleId {
        THEME_MODULE_ID
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        let service = if self.include_defaults {
            ThemeService::new().with_default_themes()
        } else {
            ThemeService::new()
        };
        if let Some(id) = &self.active {
            service.set_active_theme(id)?;
        }
        app.register_service(service)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ThemeService;
    use nest_core::AppBuilder;
    use nest_design::ThemeId;

    #[test]
    fn module_registers_theme_service() {
        let built = AppBuilder::new()
            .module(ThemeModule::default())
            .build()
            .unwrap();
        let themes = built.context.service::<ThemeService>().unwrap();
        assert_eq!(themes.list_themes().len(), 4);
        assert_eq!(themes.active_id().unwrap().as_str(), "cbre-light");
    }

    #[test]
    fn with_active_overrides_default() {
        let built = AppBuilder::new()
            .module(ThemeModule::default().with_active("cursor-dark"))
            .build()
            .unwrap();
        let themes = built.context.service::<ThemeService>().unwrap();
        assert_eq!(themes.active_id().unwrap().as_str(), "cursor-dark");
    }

    #[test]
    fn module_without_defaults_starts_empty() {
        let built = AppBuilder::new()
            .module(ThemeModule::without_defaults())
            .build()
            .unwrap();
        let themes = built.context.service::<ThemeService>().unwrap();
        assert!(themes.list_themes().is_empty());
        assert!(themes.active_id().is_err());
    }

    #[test]
    fn set_active_via_context_service() {
        let built = AppBuilder::new()
            .module(ThemeModule::default())
            .build()
            .unwrap();
        let themes = built.context.service::<ThemeService>().unwrap();
        themes
            .set_active_theme(&ThemeId::from("nest-dark"))
            .unwrap();
        assert_eq!(themes.active_theme().unwrap().id.as_str(), "nest-dark");
    }
}
