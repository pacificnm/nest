//! Optional Nest module that registers [`crate::ThemeService`].

use nest_core::{AppBuilder, Module, ModuleId, NestResult};

use crate::service::ThemeService;

/// Module id for [`ThemeModule`].
pub const THEME_MODULE_ID: ModuleId = ModuleId("nest-theme");

/// Registers [`ThemeService`] with optional built-in light/dark themes.
pub struct ThemeModule {
    include_defaults: bool,
}

impl ThemeModule {
    /// Creates a module that registers built-in themes by default.
    pub fn new() -> Self {
        Self {
            include_defaults: true,
        }
    }

    /// Creates a module without pre-registering built-in themes.
    pub fn without_defaults() -> Self {
        Self {
            include_defaults: false,
        }
    }

    /// Sets whether built-in Nest light/dark themes are registered.
    pub fn with_defaults(mut self, include_defaults: bool) -> Self {
        self.include_defaults = include_defaults;
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
        assert_eq!(themes.list_themes().len(), 2);
        assert_eq!(themes.active_id().unwrap().as_str(), "nest-light");
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
