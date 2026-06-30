# nest-validation

UI-agnostic validation for the [Nest framework](../../README.md).

**Crate path:** [`core/crates/nest-validation`](../../core/crates/nest-validation)

## Role

nest-validation owns validation logic and structured issues. Hosts decide how to present them:

| Host | Presentation |
|------|----------------|
| nest-forms (future) | Inline field errors |
| nest-api (future) | HTTP 400 + JSON issues |
| nest-cli (future) | Terminal diagnostics |
| nest-import (future) | Row/column errors |
| nest-logging | Structured failure logs |

## Quick start

```rust
use nest_core::AppBuilder;
use nest_validation::{
    validate, ValidationContext, ValidationIssue, Validate, ValidationError,
    ValidationModule, ValidatorRegistry,
};

AppBuilder::new()
    .module(ValidationModule::default())
    .build()?;
```

### Object-level validation

```rust
impl Validate for Project {
    fn validate(&self, ctx: &ValidationContext) -> nest_validation::ValidationResult {
        let mut issues = Vec::new();
        if self.name.trim().is_empty() {
            issues.push(ValidationIssue::field_error(
                "name",
                ctx.qualify_code("validation.required"),
                "Project name is required",
            ));
        }
        ValidationError::from_issues(issues)
    }
}
```

### Extending the registry

```rust
use nest_core::{AppBuilder, Module, ModuleId, NestResult};
use nest_validation::{VALIDATION_MODULE_ID, Validator, ValidatorName, ValidatorRegistry};

struct CustomerModule;

impl Module for CustomerModule {
    fn id(&self) -> ModuleId {
        ModuleId("my-app-customer")
    }

    fn dependencies(&self) -> &'static [ModuleId] {
        &[VALIDATION_MODULE_ID]
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        app.service_mut::<ValidatorRegistry>()?
            .register(CustomerNumberValidator)?;
        Ok(())
    }
}
```

## Module integration

`ValidationModule` registers `ValidatorRegistry` as a singleton service. nest-core stays validation-agnostic.

Dependency direction:

```
nest-core  ←  nest-validation
```

## Documentation

| Document | Description |
|----------|-------------|
| [Built-in validators](built-in-validators.md) | Registry validators and typed helpers |

## Related

- [nest-theme](../nest-theme/README.md) — `ThemeValidator` is separate (theme tokens only)
- [nest-core modules](../nest-core/modules.md) — `ModuleId`, dependencies, `service_mut`
