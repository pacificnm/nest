//! Safe path resolution.

use std::path::{Component, Path, PathBuf};

use crate::codes::{
    NEST_FILE_ABSOLUTE_PATH_DENIED, NEST_FILE_EMPTY_PATH, NEST_FILE_PARENT_NOT_FOUND,
    NEST_FILE_PATH_OUTSIDE_ROOT, NEST_FILE_PATH_TRAVERSAL_DENIED,
    NEST_FILE_SYMLINK_ESCAPE_DENIED,
};
use crate::error::{map_io_error, FileError, FileResult};

/// Resolves user paths against an optional root with traversal and symlink checks.
#[derive(Debug, Clone)]
pub struct SafePathResolver {
    root: Option<PathBuf>,
    allow_absolute: bool,
    allow_symlink_escape: bool,
}

impl SafePathResolver {
    /// Creates a resolver from service configuration.
    pub fn new(
        root: Option<PathBuf>,
        allow_absolute: bool,
        allow_symlink_escape: bool,
    ) -> Self {
        Self {
            root,
            allow_absolute,
            allow_symlink_escape,
        }
    }

    /// Resolves a path for read or metadata operations.
    pub fn resolve(&self, input: impl AsRef<Path>) -> FileResult<PathBuf> {
        let logical = self.resolve_logical(input.as_ref())?;
        self.finalize_path(&logical, false)
    }

    /// Resolves a path for write operations.
    pub fn resolve_for_write(
        &self,
        input: impl AsRef<Path>,
        create_parents: bool,
    ) -> FileResult<PathBuf> {
        let logical = self.resolve_logical(input.as_ref())?;
        if create_parents {
            if let Some(parent) = logical.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent).map_err(|error| {
                        map_io_error(error, parent).with_path(&logical)
                    })?;
                }
            }
        } else if let Some(parent) = logical.parent() {
            if !parent.exists() {
                return Err(
                    FileError::invalid_path(format!("parent directory not found: {}", parent.display()))
                        .with_code(NEST_FILE_PARENT_NOT_FOUND)
                        .with_path(&logical),
                );
            }
        }
        self.finalize_path(&logical, true)
    }

    fn resolve_logical(&self, input: &Path) -> FileResult<PathBuf> {
        validate_path_input(input)?;

        if input.is_absolute() {
            if self.root.is_some() && !self.allow_absolute {
                return Err(
                    FileError::invalid_path("absolute paths are denied in scoped mode")
                        .with_code(NEST_FILE_ABSOLUTE_PATH_DENIED)
                        .with_path(input),
                );
            }
            if self.root.is_none() && !self.allow_absolute {
                return Err(
                    FileError::invalid_path("absolute paths are denied")
                        .with_code(NEST_FILE_ABSOLUTE_PATH_DENIED)
                        .with_path(input),
                );
            }
        }

        match &self.root {
            Some(root) => normalize_under_root(root, input),
            None => {
                if input.is_absolute() {
                    Ok(input.to_path_buf())
                } else {
                    Ok(std::env::current_dir()
                        .map_err(|error| map_io_error(error, input))?
                        .join(input))
                }
            }
        }
    }

    fn finalize_path(&self, logical: &Path, _for_write: bool) -> FileResult<PathBuf> {
        let Some(root) = &self.root else {
            if logical.exists() {
                return logical.canonicalize().map_err(|error| map_io_error(error, logical));
            }
            return Ok(logical.to_path_buf());
        };

        let canonical_root = fs_canonicalize(root, root)?;

        if logical.exists() {
            let canonical = fs_canonicalize(logical, logical)?;
            return ensure_within_root(&canonical, &canonical_root, self.allow_symlink_escape);
        }

        let mut ancestor = logical.to_path_buf();
        loop {
            if ancestor.exists() {
                let canonical_ancestor = fs_canonicalize(&ancestor, &ancestor)?;
                ensure_within_root(&canonical_ancestor, &canonical_root, self.allow_symlink_escape)?;

                let suffix = logical
                    .strip_prefix(&ancestor)
                    .unwrap_or_else(|_| Path::new(""));
                let resolved = canonical_ancestor.join(suffix);
                return ensure_within_root(&resolved, &canonical_root, self.allow_symlink_escape);
            }

            if ancestor == *root {
                break;
            }

            match ancestor.parent() {
                Some(parent) => ancestor = parent.to_path_buf(),
                None => break,
            }
        }

        if logical.starts_with(root) {
            Ok(logical.to_path_buf())
        } else {
            Err(
                FileError::invalid_path(format!("path is outside root: {}", logical.display()))
                    .with_code(NEST_FILE_PATH_OUTSIDE_ROOT)
                    .with_path(logical),
            )
        }
    }
}

fn validate_path_input(input: &Path) -> FileResult<()> {
    if input.as_os_str().is_empty() {
        return Err(FileError::invalid_path("path must not be empty").with_code(NEST_FILE_EMPTY_PATH));
    }

    let bytes = input.as_os_str().as_encoded_bytes();
    if bytes.contains(&0) {
        return Err(
            FileError::invalid_path("path contains NUL byte").with_code(NEST_FILE_EMPTY_PATH),
        );
    }

    Ok(())
}

fn normalize_under_root(root: &Path, relative: &Path) -> FileResult<PathBuf> {
    if relative.is_absolute() {
        return Ok(relative.to_path_buf());
    }

    let mut result = root.to_path_buf();
    for component in relative.components() {
        match component {
            Component::Normal(part) => result.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                if !result.pop() {
                    return Err(
                        FileError::invalid_path("path traversal denied")
                            .with_code(NEST_FILE_PATH_TRAVERSAL_DENIED)
                            .with_path(relative),
                    );
                }
                if !result.starts_with(root) {
                    return Err(
                        FileError::invalid_path("path traversal denied")
                            .with_code(NEST_FILE_PATH_TRAVERSAL_DENIED)
                            .with_path(relative),
                    );
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(
                    FileError::invalid_path("absolute path components are denied in scoped mode")
                        .with_code(NEST_FILE_ABSOLUTE_PATH_DENIED)
                        .with_path(relative),
                );
            }
        }
    }

    if !result.starts_with(root) {
        return Err(
            FileError::invalid_path("path is outside root")
                .with_code(NEST_FILE_PATH_OUTSIDE_ROOT)
                .with_path(relative),
        );
    }

    Ok(result)
}

fn fs_canonicalize(path: &Path, context: &Path) -> FileResult<PathBuf> {
    path.canonicalize()
        .map_err(|error| map_io_error(error, context))
}

fn ensure_within_root(
    path: &Path,
    canonical_root: &Path,
    allow_symlink_escape: bool,
) -> FileResult<PathBuf> {
    if path.starts_with(canonical_root) {
        return Ok(path.to_path_buf());
    }

    if allow_symlink_escape {
        return Err(
            FileError::invalid_path(format!("path is outside root: {}", path.display()))
                .with_code(NEST_FILE_PATH_OUTSIDE_ROOT)
                .with_path(path),
        );
    }

    Err(
        FileError::invalid_path(format!("symlink escape denied: {}", path.display()))
            .with_code(NEST_FILE_SYMLINK_ESCAPE_DENIED)
            .with_path(path),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn scoped_rejects_parent_traversal() {
        let dir = tempdir().unwrap();
        let resolver = SafePathResolver::new(Some(dir.path().to_path_buf()), false, false);
        let err = resolver.resolve("../outside.txt").unwrap_err();
        assert_eq!(err.code(), Some(NEST_FILE_PATH_TRAVERSAL_DENIED));
    }

    #[test]
    fn scoped_rejects_absolute_path() {
        let dir = tempdir().unwrap();
        let resolver = SafePathResolver::new(Some(dir.path().to_path_buf()), false, false);
        let err = resolver.resolve("/etc/passwd").unwrap_err();
        assert_eq!(err.code(), Some(NEST_FILE_ABSOLUTE_PATH_DENIED));
    }

    #[test]
    fn scoped_allows_relative_file() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("docs/readme.md");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, "hello").unwrap();

        let resolver = SafePathResolver::new(Some(dir.path().to_path_buf()), false, false);
        let resolved = resolver.resolve("docs/readme.md").unwrap();
        assert!(resolved.exists());
    }

    #[cfg(unix)]
    #[test]
    fn scoped_rejects_symlink_escape() {
        use std::os::unix::fs::symlink;

        let dir = tempdir().unwrap();
        let outside = tempdir().unwrap();
        let outside_file = outside.path().join("secret.txt");
        fs::write(&outside_file, "secret").unwrap();

        let link_path = dir.path().join("link");
        symlink(outside.path(), &link_path).unwrap();

        let resolver = SafePathResolver::new(Some(dir.path().to_path_buf()), false, false);
        let err = resolver.resolve("link/secret.txt").unwrap_err();
        assert_eq!(err.code(), Some(NEST_FILE_SYMLINK_ESCAPE_DENIED));
    }
}
