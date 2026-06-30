# Built-in validators

## Registry validators (string fields)

Registered by default via `ValidationModule::default()`:

| Name | Type | Description |
|------|------|-------------|
| `required` | `RequiredValidator` | Non-empty after trim |
| `not_empty` | `NotEmptyValidator` | Any non-zero length |
| `email` | `EmailValidator` | Basic `@` + domain shape |
| `url` | `UrlValidator` | `http://` or `https://` prefix |
| `min_length` | `MinLengthValidator::default()` | Min 1 char (customize with `::new(n)`) |
| `max_length` | `MaxLengthValidator::default()` | Max 255 chars (customize with `::new(n)`) |

### Parametric validators

```rust
registry.register(MinLengthValidator::named("password_min", 8))?;
registry.register(OneOfValidator::new(["draft", "published"]))?;
```

### Regex (optional feature)

Enable `nest-validation/regex`:

```rust
registry.register(RegexValidator::new(r"^\d{4}$")?)?;
```

## Typed validators (not in registry by default)

Use directly in `Validate` impls or via `merge_issues`:

| Validator | Type param | Description |
|-----------|------------|-------------|
| `RangeValidator` | `T: PartialOrd` | Inclusive numeric range |
| `MinLengthValidator` / `MaxLengthValidator` | string via `NamedValidator` | Length bounds |

## Object-level rules

Cross-field rules (e.g. start date before end date) belong on `Validate` for the parent type, not the registry.

## Error codes

Issues use codes like `validation.email`, `validation.required`. Prefix with `ValidationContext::with_key_prefix("form.")` when needed.

Nest-level conversion uses `NEST_VALIDATION_FAILED` when converting `ValidationError` to `NestError`.
