# nest-validation v1 Implementation Plan

## Status: Implemented

See [nest-validation docs](../nest-validation/README.md).

## Context

UI/host-agnostic validation foundation crate. Structured issues for forms, API, CLI, and imports. Integrates with nest-core via optional `ValidationModule` — same pattern as `nest-theme`.

## Crate boundaries

| Crate | Role |
|-------|------|
| `nest-validation` | `Validate`, `Validator`, `ValidatorRegistry`, built-ins, `ValidationModule` |
| `nest-core` | `ModuleId`, `dependencies()`, topo-sort configure, `service_mut` |
| `nest-error` | `NEST_MODULE_DEPENDENCY_MISSING`, `From<ValidationError>` in nest-validation only |

## nest-core extensions

- `Module::id()` + `Module::dependencies()`
- `configure` returns `NestResult<()>`
- Modules configured at `build()` in dependency order
- `AppBuilder::service_mut` for configure-time extension

## nest-validation

- Collect-all issues; blocking on `Severity::Error` only
- `ValidatorRegistry` for named string-field validators
- Built-ins: required, not_empty, email, url, min_length, max_length, range, one_of; regex behind feature
- Date range: object-level `Validate` only (no chrono)

## v1 limitations

- No `nest-validation-derive`
- No platform hosts (nest-forms, nest-api, nest-cli)
- Configure-time registry only (no post-build mutation)
- `nest-theme::ThemeValidator` remains separate

## Follow-up

- `nest-validation-derive` / `#[derive(Validate)]`
- Platform hosts consuming `ValidationIssue`
- JSON schema export for nest-react
