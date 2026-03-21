use std::path::{Path, PathBuf};

/// Errors from install directory resolution.
#[derive(Debug, thiserror::Error)]
pub enum ResolveError {
    #[error("resolving home directory: {0}")]
    HomeDir(String),
    #[error("resolving install dir for scope {scope:?}: {source}")]
    Scope {
        scope: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("not in a git repository")]
    NotInRepo,
    #[error(".git in {0} is a symlink, which is not supported for security reasons")]
    SymlinkGit(PathBuf),
    #[error(".git in {0} has unsupported file type")]
    UnsupportedGitType(PathBuf),
    #[error("unknown scope {0:?} (supported: user, repo)")]
    UnknownScope(String),
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

/// Return the skill installation directory for the given scope.
///
/// - `None` or `"user"` → `~/.agents/skills`
/// - `"repo"` → `<repo-root>/.agents/skills`
pub fn install_dir_for_scope(scope: Option<&str>) -> Result<PathBuf, ResolveError> {
    match scope.unwrap_or("user") {
        "user" | "" => {
            let home = home_dir()?;
            Ok(home.join(".agents").join("skills"))
        }
        "repo" => {
            let wd = std::env::current_dir()?;
            let root = find_repo_root(&wd)?;
            Ok(root.join(".agents").join("skills"))
        }
        other => Err(ResolveError::UnknownScope(other.to_string())),
    }
}

/// Resolve the effective installation directory.
/// `--prefix` takes precedence over `--scope`.
pub fn install_dir(prefix: Option<&str>, scope: Option<&str>) -> Result<PathBuf, ResolveError> {
    if let Some(p) = prefix {
        return Ok(PathBuf::from(p));
    }
    install_dir_for_scope(scope)
}

/// Traverse parent directories to find the repository root (directory containing `.git`).
///
/// Uses `symlink_metadata` to avoid following symlinks. A symlink named `.git`
/// is treated as a hard error for security.
fn find_repo_root(start_dir: &Path) -> Result<PathBuf, ResolveError> {
    let mut dir = start_dir.canonicalize().map_err(|e| ResolveError::Scope {
        scope: "repo".to_string(),
        source: Box::new(e),
    })?;

    loop {
        let git_path = dir.join(".git");
        match std::fs::symlink_metadata(&git_path) {
            Ok(meta) => {
                let ft = meta.file_type();
                if ft.is_dir() || ft.is_file() {
                    return Ok(dir);
                }
                if ft.is_symlink() {
                    return Err(ResolveError::SymlinkGit(dir));
                }
                return Err(ResolveError::UnsupportedGitType(dir));
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Continue searching upward.
            }
            Err(e) => {
                return Err(ResolveError::Scope {
                    scope: "repo".to_string(),
                    source: Box::new(e),
                });
            }
        }

        let parent = dir.parent().map(|p| p.to_path_buf());
        match parent {
            Some(p) if p != dir => dir = p,
            _ => return Err(ResolveError::NotInRepo),
        }
    }
}

fn home_dir() -> Result<PathBuf, ResolveError> {
    std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| ResolveError::HomeDir("HOME environment variable not set".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_install_dir_user_scope() {
        let dir = install_dir_for_scope(Some("user")).unwrap();
        assert!(dir.ends_with(".agents/skills"));
    }

    #[test]
    fn test_install_dir_default_scope() {
        let dir = install_dir_for_scope(None).unwrap();
        assert!(dir.ends_with(".agents/skills"));
    }

    #[test]
    fn test_install_dir_unknown_scope() {
        let err = install_dir_for_scope(Some("invalid")).unwrap_err();
        assert!(matches!(err, ResolveError::UnknownScope(_)));
    }

    #[test]
    fn test_prefix_overrides_scope() {
        let dir = install_dir(Some("/custom/path"), Some("repo")).unwrap();
        assert_eq!(dir, PathBuf::from("/custom/path"));
    }

    #[test]
    fn test_find_repo_root() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join(".git")).unwrap();
        let sub = tmp.path().join("a").join("b");
        std::fs::create_dir_all(&sub).unwrap();

        let root = find_repo_root(&sub).unwrap();
        assert_eq!(root, tmp.path().canonicalize().unwrap());
    }

    #[test]
    fn test_find_repo_root_not_found() {
        let tmp = TempDir::new().unwrap();
        let err = find_repo_root(tmp.path()).unwrap_err();
        assert!(matches!(err, ResolveError::NotInRepo));
    }
}
