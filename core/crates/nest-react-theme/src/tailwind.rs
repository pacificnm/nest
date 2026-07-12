//! Tailwind CSS preset referencing Nest CSS variables.

/// Returns a Tailwind v3 preset JSON document that maps `nest-*` utilities to
/// `--nest-*` CSS custom properties produced by [`crate::CssTheme`].
pub fn tailwind_preset_json() -> &'static str {
    r#"{
  "theme": {
    "extend": {
      "colors": {
        "nest-background": "var(--nest-color-background)",
        "nest-foreground": "var(--nest-color-foreground)",
        "nest-primary": "var(--nest-color-primary)",
        "nest-secondary": "var(--nest-color-secondary)",
        "nest-border": "var(--nest-color-border)",
        "nest-surface": "var(--nest-color-surface)",
        "nest-accent": "var(--nest-color-accent)",
        "nest-muted": "var(--nest-color-muted)",
        "nest-success": "var(--nest-color-success)",
        "nest-warning": "var(--nest-color-warning)",
        "nest-error": "var(--nest-color-error)",
        "nest-info": "var(--nest-color-info)"
      },
      "spacing": {
        "nest-xs": "var(--nest-spacing-xs)",
        "nest-sm": "var(--nest-spacing-sm)",
        "nest-md": "var(--nest-spacing-md)",
        "nest-lg": "var(--nest-spacing-lg)",
        "nest-xl": "var(--nest-spacing-xl)",
        "nest-xxl": "var(--nest-spacing-xxl)"
      },
      "borderRadius": {
        "nest-sm": "var(--nest-radius-sm)",
        "nest-md": "var(--nest-radius-md)",
        "nest-lg": "var(--nest-radius-lg)",
        "nest-full": "var(--nest-radius-full)"
      }
    }
  }
}"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_is_valid_json() {
        let _: serde_json::Value = serde_json::from_str(tailwind_preset_json()).unwrap();
    }

    /// Guards against drift: every committed `nest-tailwind-preset.json` in the repo
    /// (the copies each app's `tailwind.config.ts` imports) must match this function,
    /// the source of truth. If this fails, regenerate the offending copy from
    /// `tailwind_preset_json()`.
    #[test]
    fn committed_preset_copies_match_source() {
        use std::fs;
        use std::path::{Path, PathBuf};

        let source: serde_json::Value =
            serde_json::from_str(tailwind_preset_json()).expect("source preset is valid JSON");

        // core/crates/nest-react-theme -> repo root is three levels up.
        let repo_root = match PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(3) {
            Some(root) => root.to_path_buf(),
            None => return, // unusual layout; nothing to check.
        };

        fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
            let Ok(entries) = fs::read_dir(dir) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if matches!(name, "node_modules" | "target" | ".git") {
                        continue;
                    }
                    collect(&path, out);
                } else if path.file_name().and_then(|n| n.to_str())
                    == Some("nest-tailwind-preset.json")
                {
                    out.push(path);
                }
            }
        }

        let mut copies = Vec::new();
        collect(&repo_root, &mut copies);

        for copy in copies {
            let text = fs::read_to_string(&copy).expect("read preset copy");
            let value: serde_json::Value = serde_json::from_str(&text)
                .unwrap_or_else(|e| panic!("{} is not valid JSON: {e}", copy.display()));
            assert_eq!(
                value,
                source,
                "{} is out of sync with nest_react_theme::tailwind_preset_json(); \
                 regenerate it from the source",
                copy.display()
            );
        }
    }
}
