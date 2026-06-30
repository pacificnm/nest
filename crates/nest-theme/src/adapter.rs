//! Platform adapter contract for converting themes to host-specific output.

use nest_design::ThemeDefinition;
use nest_error::NestResult;

/// Converts a [`ThemeDefinition`] into a platform-specific representation.
///
/// Implemented by adapter crates such as `nest-egui-theme`, `nest-react-theme`,
/// and `nest-cli-theme`.
pub trait ThemeAdapter<TOutput> {
    /// Adapts the theme into the host output type.
    fn adapt(theme: &ThemeDefinition) -> NestResult<TOutput>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use nest_design::themes::light;

    struct StubAdapter;

    impl ThemeAdapter<String> for StubAdapter {
        fn adapt(theme: &ThemeDefinition) -> NestResult<String> {
            Ok(format!("theme:{}", theme.id.as_str()))
        }
    }

    #[test]
    fn stub_adapter_compiles_and_runs() {
        let theme = light();
        let output = StubAdapter::adapt(&theme).unwrap();
        assert_eq!(output, "theme:nest-light");
    }
}
