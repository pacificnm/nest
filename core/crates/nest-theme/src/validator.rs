//! Theme validation against required token sets.

use nest_design::{ColorToken, ThemeDefinition};

use crate::error::{self, ThemeResult};

/// Validates a [`ThemeDefinition`] before registration.
pub struct ThemeValidator;

impl ThemeValidator {
    /// Validates all required tokens and color formats.
    pub fn validate(theme: &ThemeDefinition) -> ThemeResult<()> {
        if theme.id.as_str().is_empty() {
            return Err(error::validation("id", "theme id must not be empty"));
        }

        validate_color(&theme.colors.background, "colors.background")?;
        validate_color(&theme.colors.foreground, "colors.foreground")?;
        validate_color(&theme.colors.primary, "colors.primary")?;
        validate_color(&theme.colors.secondary, "colors.secondary")?;
        validate_color(&theme.colors.border, "colors.border")?;
        validate_color(&theme.colors.surface, "colors.surface")?;

        if let Some(accent) = &theme.colors.accent {
            validate_color(accent, "colors.accent")?;
        }
        if let Some(muted) = &theme.colors.muted {
            validate_color(muted, "colors.muted")?;
        }

        validate_spacing(&theme.spacing, "spacing")?;
        validate_typography(&theme.typography.body, "typography.body")?;
        validate_typography(&theme.typography.heading, "typography.heading")?;

        if let Some(caption) = &theme.typography.caption {
            validate_typography(caption, "typography.caption")?;
        }
        if let Some(mono) = &theme.typography.mono {
            validate_typography(mono, "typography.mono")?;
        }

        validate_color(&theme.status.success, "status.success")?;
        validate_color(&theme.status.warning, "status.warning")?;
        validate_color(&theme.status.error, "status.error")?;
        validate_color(&theme.status.info, "status.info")?;

        Ok(())
    }
}

fn validate_color(token: &ColorToken, field: &str) -> ThemeResult<()> {
    token
        .validate()
        .map_err(|e| error::validation(field, format!("invalid color {:?}: {e}", token.as_str())))
}

fn validate_spacing(spacing: &nest_design::SpacingTokens, field: &str) -> ThemeResult<()> {
    for (name, value) in [
        ("xs", spacing.xs),
        ("sm", spacing.sm),
        ("md", spacing.md),
        ("lg", spacing.lg),
        ("xl", spacing.xl),
    ] {
        if value < 0.0 {
            return Err(error::validation(
                &format!("{field}.{name}"),
                "spacing value must be non-negative",
            ));
        }
    }
    Ok(())
}

fn validate_typography(style: &nest_design::TypographyStyle, field: &str) -> ThemeResult<()> {
    if style.font_family.is_empty() {
        return Err(error::validation(
            &format!("{field}.font_family"),
            "font_family must not be empty",
        ));
    }
    if style.size <= 0.0 {
        return Err(error::validation(
            &format!("{field}.size"),
            "size must be positive",
        ));
    }
    if style.line_height <= 0.0 {
        return Err(error::validation(
            &format!("{field}.line_height"),
            "line_height must be positive",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nest_design::themes::{dark, light};

    #[test]
    fn builtin_themes_validate() {
        ThemeValidator::validate(&light()).unwrap();
        ThemeValidator::validate(&dark()).unwrap();
    }

    #[test]
    fn empty_id_fails() {
        let mut theme = light();
        theme.id = nest_design::ThemeId::new("");
        let err = ThemeValidator::validate(&theme).unwrap_err();
        assert_eq!(err.kind(), nest_error::NestErrorKind::Validation);
    }

    #[test]
    fn invalid_color_fails_with_field() {
        let mut theme = light();
        theme.colors.primary = ColorToken("#BAD".to_string());
        let err = ThemeValidator::validate(&theme).unwrap_err();
        assert_eq!(err.kind(), nest_error::NestErrorKind::Validation);
        assert!(err.message().contains("colors.primary"));
    }
}
