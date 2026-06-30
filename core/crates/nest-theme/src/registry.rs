//! In-memory theme registry.

use std::collections::HashMap;

use nest_design::{ThemeDefinition, ThemeId};

use crate::error::{theme_already_registered, theme_not_found, ThemeResult};
use crate::validator::ThemeValidator;

/// Stores registered themes keyed by id.
#[derive(Debug, Default)]
pub struct ThemeRegistry {
    themes: HashMap<ThemeId, ThemeDefinition>,
}

impl ThemeRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self {
            themes: HashMap::new(),
        }
    }

    /// Registers a theme after validation. Errors on duplicate ids.
    pub fn register(&mut self, theme: ThemeDefinition) -> ThemeResult<()> {
        ThemeValidator::validate(&theme)?;
        let id = theme.id.clone();
        if self.themes.contains_key(&id) {
            return Err(theme_already_registered(id.as_str()));
        }
        self.themes.insert(id, theme);
        Ok(())
    }

    /// Returns a reference to a registered theme.
    pub fn get(&self, id: &ThemeId) -> ThemeResult<&ThemeDefinition> {
        self.themes
            .get(id)
            .ok_or_else(|| theme_not_found(id.as_str()))
    }

    /// Returns whether a theme id is registered.
    pub fn contains(&self, id: &ThemeId) -> bool {
        self.themes.contains_key(id)
    }

    /// Returns all registered theme ids.
    pub fn ids(&self) -> impl Iterator<Item = &ThemeId> {
        self.themes.keys()
    }

    /// Returns the number of registered themes.
    pub fn len(&self) -> usize {
        self.themes.len()
    }

    /// Returns whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.themes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nest_design::themes::{dark, light};

    #[test]
    fn register_and_get() {
        let mut registry = ThemeRegistry::new();
        registry.register(light()).unwrap();
        let theme = registry.get(&ThemeId::from("nest-light")).unwrap();
        assert_eq!(theme.id.as_str(), "nest-light");
    }

    #[test]
    fn duplicate_registration_fails() {
        let mut registry = ThemeRegistry::new();
        registry.register(light()).unwrap();
        let err = registry.register(light()).unwrap_err();
        assert_eq!(
            err.code(),
            Some(crate::codes::NEST_THEME_ALREADY_REGISTERED)
        );
    }

    #[test]
    fn missing_theme_fails() {
        let registry = ThemeRegistry::new();
        let err = registry.get(&ThemeId::from("missing")).unwrap_err();
        assert_eq!(err.code(), Some(crate::codes::NEST_THEME_NOT_FOUND));
    }

    #[test]
    fn ids_lists_registered() {
        let mut registry = ThemeRegistry::new();
        registry.register(light()).unwrap();
        registry.register(dark()).unwrap();
        let ids: Vec<_> = registry.ids().map(|id| id.as_str().to_string()).collect();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"nest-light".to_string()));
        assert!(ids.contains(&"nest-dark".to_string()));
    }
}
