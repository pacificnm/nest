//! Locates the Nest repository root for shell services.

use std::path::PathBuf;

/// Locates the Nest repository root (`Cargo.toml` workspace + `apps/`).
pub fn resolve_nest_root() -> Result<PathBuf, String> {
    let mut dir = std::env::current_dir().map_err(|error| error.to_string())?;
    for _ in 0..8 {
        if dir.join("apps").is_dir() && dir.join("Cargo.toml").is_file() {
            return Ok(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    Err("could not locate Nest repository root (expected apps/ and Cargo.toml)".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_nest_root_from_ui_directory() {
        let nest_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let previous = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(&nest_root).expect("chdir nest root");
        let root = resolve_nest_root().expect("nest root");
        assert!(root.join("apps").is_dir());
        std::env::set_current_dir(previous).expect("restore cwd");
    }
}
