//! Active theme state and registration API.

use std::path::Path;
use std::sync::RwLock;

use nest_design::{ThemeDefinition, ThemeId};

use crate::error::{no_active_theme, theme_not_found, ThemeResult};
use crate::loader::ThemeLoader;
use crate::registry::ThemeRegistry;

/// Runtime theme service: registry, loading, validation, and active theme state.
///
/// Registered as a singleton via [`crate::ThemeModule`]. Uses interior mutability
/// because [`nest_core::AppContext`] exposes services as shared references.
pub struct ThemeService {
    registry: RwLock<ThemeRegistry>,
    active: RwLock<Option<ThemeId>>,
}

impl ThemeService {
    /// Creates an empty theme service with no registered themes.
    pub fn new() -> Self {
        Self {
            registry: RwLock::new(ThemeRegistry::new()),
            active: RwLock::new(None),
        }
    }

    /// Registers the built-in Nest light and dark themes and sets `nest-light` active.
    pub fn with_default_themes(self) -> Self {
        self.register_theme(nest_design::themes::light())
            .expect("built-in light theme is valid");
        self.register_theme(nest_design::themes::dark())
            .expect("built-in dark theme is valid");
        *self.active.write().expect("theme lock") = Some(ThemeId::from("nest-light"));
        self
    }

    /// Registers a theme after validation.
    pub fn register_theme(&self, theme: ThemeDefinition) -> ThemeResult<()> {
        self.registry
            .write()
            .expect("theme registry lock")
            .register(theme)
    }

    /// Loads a theme from disk, registers it, and returns its id.
    pub fn load_and_register(&self, path: impl AsRef<Path>) -> ThemeResult<ThemeId> {
        let theme = ThemeLoader::from_file(path)?;
        let id = theme.id.clone();
        self.register_theme(theme)?;
        Ok(id)
    }

    /// Sets the active theme by id.
    pub fn set_active_theme(&self, id: &ThemeId) -> ThemeResult<()> {
        if !self
            .registry
            .read()
            .expect("theme registry lock")
            .contains(id)
        {
            return Err(theme_not_found(id.as_str()));
        }
        *self.active.write().expect("active theme lock") = Some(id.clone());
        Ok(())
    }

    /// Returns a clone of the active theme definition.
    pub fn active_theme(&self) -> ThemeResult<ThemeDefinition> {
        let id = self.active_id()?;
        self.theme(&id)
    }

    /// Returns the active theme id.
    pub fn active_id(&self) -> ThemeResult<ThemeId> {
        self.active
            .read()
            .expect("active theme lock")
            .clone()
            .ok_or_else(no_active_theme)
    }

    /// Returns a clone of a registered theme by id.
    pub fn theme(&self, id: &ThemeId) -> ThemeResult<ThemeDefinition> {
        let registry = self.registry.read().expect("theme registry lock");
        Ok(registry.get(id)?.clone())
    }

    /// Lists all registered theme ids.
    pub fn list_themes(&self) -> Vec<ThemeId> {
        self.registry
            .read()
            .expect("theme registry lock")
            .ids()
            .cloned()
            .collect()
    }
}

impl Default for ThemeService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codes;

    #[test]
    fn default_themes_and_active_round_trip() {
        let service = ThemeService::new().with_default_themes();
        assert_eq!(service.list_themes().len(), 2);
        assert_eq!(service.active_id().unwrap().as_str(), "nest-light");

        service
            .set_active_theme(&ThemeId::from("nest-dark"))
            .unwrap();
        let active = service.active_theme().unwrap();
        assert_eq!(active.id.as_str(), "nest-dark");
        assert_eq!(active.mode, nest_design::ThemeMode::Dark);
    }

    #[test]
    fn set_unknown_theme_fails() {
        let service = ThemeService::new().with_default_themes();
        let err = service
            .set_active_theme(&ThemeId::from("missing"))
            .unwrap_err();
        assert_eq!(err.code(), Some(codes::NEST_THEME_NOT_FOUND));
    }

    #[test]
    fn duplicate_registration_fails() {
        let service = ThemeService::new();
        service
            .register_theme(nest_design::themes::light())
            .unwrap();
        let err = service
            .register_theme(nest_design::themes::light())
            .unwrap_err();
        assert_eq!(err.code(), Some(codes::NEST_THEME_ALREADY_REGISTERED));
    }

    #[test]
    fn no_active_theme_when_empty() {
        let service = ThemeService::new();
        let err = service.active_theme().unwrap_err();
        assert_eq!(err.code(), Some(codes::NEST_THEME_NO_ACTIVE));
    }

    #[test]
    fn load_and_register_from_toml() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../themes/nest-dark.toml");
        let service = ThemeService::new();
        let id = service.load_and_register(&path).unwrap();
        assert_eq!(id.as_str(), "nest-dark");
        service.set_active_theme(&id).unwrap();
        assert_eq!(service.active_id().unwrap(), id);
    }
}
