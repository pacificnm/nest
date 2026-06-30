//! Optional Nest module that registers [`crate::ValidatorRegistry`].

use nest_core::{AppBuilder, Module, ModuleId, NestResult};

use crate::builtins::register_defaults;
use crate::registry::ValidatorRegistry;

/// Module id for [`ValidationModule`].
pub const VALIDATION_MODULE_ID: ModuleId = ModuleId("nest-validation");

/// Registers [`ValidatorRegistry`] with optional built-in validators.
pub struct ValidationModule {
    include_defaults: bool,
}

impl ValidationModule {
    /// Creates a module that registers built-in validators by default.
    pub fn new() -> Self {
        Self {
            include_defaults: true,
        }
    }

    /// Creates a module without pre-registering built-in validators.
    pub fn without_defaults() -> Self {
        Self {
            include_defaults: false,
        }
    }

    /// Sets whether built-in validators are registered.
    pub fn with_defaults(mut self, include_defaults: bool) -> Self {
        self.include_defaults = include_defaults;
        self
    }
}

impl Default for ValidationModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for ValidationModule {
    fn id(&self) -> ModuleId {
        VALIDATION_MODULE_ID
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        let mut registry = ValidatorRegistry::new();
        if self.include_defaults {
            register_defaults(&mut registry)?;
        }
        app.register_service(registry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ValidatorRegistry;
    use nest_core::AppBuilder;

    struct CustomerNumberValidator;

    impl crate::validator::ValidatorName for CustomerNumberValidator {
        const NAME: &'static str = "customer_number";
    }

    impl crate::validator::Validator<str> for CustomerNumberValidator {
        fn validate(
            &self,
            value: &str,
            _ctx: &crate::ValidationContext,
        ) -> Vec<crate::ValidationIssue> {
            if value.starts_with("CUST-") {
                vec![]
            } else {
                vec![crate::ValidationIssue::error(
                    "validation.customer_number",
                    "Customer number must start with CUST-",
                )]
            }
        }
    }

    struct CustomerModule;

    impl Module for CustomerModule {
        fn id(&self) -> ModuleId {
            ModuleId("test-customer")
        }

        fn dependencies(&self) -> &'static [ModuleId] {
            &[VALIDATION_MODULE_ID]
        }

        fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
            let validators = app.service_mut::<ValidatorRegistry>()?;
            validators.register(CustomerNumberValidator)?;
            Ok(())
        }
    }

    #[test]
    fn module_registers_default_validators() {
        let built = AppBuilder::new()
            .module(ValidationModule::default())
            .build()
            .unwrap();
        let validators = built.context.service::<ValidatorRegistry>().unwrap();
        assert!(validators.contains("email"));
        assert!(validators.contains("required"));
    }

    #[test]
    fn module_without_defaults_starts_empty() {
        let built = AppBuilder::new()
            .module(ValidationModule::without_defaults())
            .build()
            .unwrap();
        let validators = built.context.service::<ValidatorRegistry>().unwrap();
        assert!(!validators.contains("email"));
    }

    #[test]
    fn dependent_module_extends_registry() {
        let built = AppBuilder::new()
            .module(CustomerModule)
            .module(ValidationModule::default())
            .build()
            .unwrap();
        let validators = built.context.service::<ValidatorRegistry>().unwrap();
        assert!(validators.contains("customer_number"));
        let issues = validators
            .validate_named("customer_number", "BAD", &crate::ValidationContext::new())
            .unwrap();
        assert_eq!(issues.len(), 1);
    }
}
