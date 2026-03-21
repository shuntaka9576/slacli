use std::path::Path;

use chrono::Utc;
use include_dir::Dir;

use crate::discover;
use crate::metadata::{self, SkillMeta};
use crate::skill::Skill;

/// Controls install/update/reinstall behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyMode {
    Install,
    Update,
    Reinstall,
}

/// Options for [`copy_skills`].
#[derive(Debug, Clone)]
pub struct CopyOptions {
    pub mode: CopyMode,
    pub force: bool,
    pub dry_run: bool,
    pub name: String,
    pub version: String,
}

/// The kind of action taken for a single skill.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionKind {
    Installed,
    Updated,
    Reinstalled,
    Skipped,
    Warned,
}

/// Describes what happened to a single skill during [`copy_skills`].
#[derive(Debug, Clone)]
pub struct SkillAction {
    pub dir: String,
    pub action: ActionKind,
    pub message: String,
}

/// Summarizes the outcome of a [`copy_skills`] call.
#[derive(Debug, Default)]
pub struct CopyResult {
    pub actions: Vec<SkillAction>,
}

impl CopyResult {
    pub fn installed(&self) -> Vec<&SkillAction> {
        self.actions
            .iter()
            .filter(|a| {
                matches!(
                    a.action,
                    ActionKind::Installed | ActionKind::Updated | ActionKind::Reinstalled
                )
            })
            .collect()
    }

    pub fn skipped(&self) -> Vec<&SkillAction> {
        self.actions
            .iter()
            .filter(|a| a.action == ActionKind::Skipped)
            .collect()
    }

    pub fn warned(&self) -> Vec<&SkillAction> {
        self.actions
            .iter()
            .filter(|a| a.action == ActionKind::Warned)
            .collect()
    }
}

/// Errors from copy operations.
#[derive(Debug, thiserror::Error)]
pub enum CopyError {
    #[error("discovering skills: {0}")]
    Discover(String),
    #[error("copying skill {skill:?}: {source}")]
    Copy {
        skill: String,
        #[source]
        source: std::io::Error,
    },
    #[error("{0}")]
    Other(String),
}

/// Copy skills from an embedded [`Dir`] into `dest_dir`.
pub fn copy_skills(
    src: &Dir<'_>,
    dest_dir: &Path,
    opts: &CopyOptions,
) -> Result<CopyResult, CopyError> {
    let (skills, skill_errors) = discover::discover(src);

    let mut result = CopyResult::default();

    // Record discovery warnings.
    for se in &skill_errors {
        result.actions.push(SkillAction {
            dir: se.dir.clone(),
            action: ActionKind::Warned,
            message: se.err.to_string(),
        });
    }

    for skill in &skills {
        let (action, message) = copy_skill(src, dest_dir, skill, opts)?;
        result.actions.push(SkillAction {
            dir: skill.dir.clone(),
            action,
            message,
        });
    }

    Ok(result)
}

/// Compare two version strings using semver when possible.
/// Returns `(cmp, true)` when both are valid semver, `(0, false)` otherwise.
fn compare_versions_safe(a: &str, b: &str) -> (std::cmp::Ordering, bool) {
    let va = ensure_v_prefix(a);
    let vb = ensure_v_prefix(b);

    let parsed_a = semver::Version::parse(va.strip_prefix('v').unwrap_or(&va));
    let parsed_b = semver::Version::parse(vb.strip_prefix('v').unwrap_or(&vb));

    match (parsed_a, parsed_b) {
        (Ok(a), Ok(b)) => (a.cmp(&b), true),
        _ => (std::cmp::Ordering::Equal, false),
    }
}

fn ensure_v_prefix(v: &str) -> String {
    if v.starts_with('v') {
        v.to_string()
    } else {
        format!("v{v}")
    }
}

fn copy_skill(
    src: &Dir<'_>,
    dest_dir: &Path,
    skill: &Skill,
    opts: &CopyOptions,
) -> Result<(ActionKind, String), CopyError> {
    let dest = dest_dir.join(&skill.dir);
    let managed = metadata::is_managed(&dest);

    // State machine per mode.
    match opts.mode {
        CopyMode::Install => {
            if dest.exists() {
                if !managed {
                    if !opts.force {
                        return Ok((
                            ActionKind::Warned,
                            format!(
                                "skill {:?} exists but is not managed by {}; use --force to overwrite",
                                skill.dir, opts.name
                            ),
                        ));
                    }
                    // Force overwrite of unmanaged skill.
                } else {
                    return Ok((
                        ActionKind::Skipped,
                        format!(
                            "skill {:?} already installed (use 'update' or 'reinstall' to refresh)",
                            skill.dir
                        ),
                    ));
                }
            }
        }
        CopyMode::Update => {
            if !managed {
                return Ok((
                    ActionKind::Skipped,
                    format!("skill {:?} is not managed by {}", skill.dir, opts.name),
                ));
            }
            if let Ok(meta) = metadata::read_meta(&dest) {
                let (cmp, ok) = compare_versions_safe(&meta.version, &opts.version);
                if ok {
                    if cmp != std::cmp::Ordering::Less {
                        return Ok((
                            ActionKind::Skipped,
                            format!(
                                "skill {:?} is already at version {:?} (>= {:?})",
                                skill.dir, meta.version, opts.version
                            ),
                        ));
                    }
                } else if meta.version == opts.version {
                    return Ok((
                        ActionKind::Skipped,
                        format!(
                            "skill {:?} is already at version {:?}",
                            skill.dir, meta.version
                        ),
                    ));
                }
            }
        }
        CopyMode::Reinstall => {
            if !managed {
                if !opts.force {
                    return Ok((
                        ActionKind::Warned,
                        format!(
                            "skill {:?} is not managed by {}; use --force to overwrite",
                            skill.dir, opts.name
                        ),
                    ));
                }
            } else if let Ok(meta) = metadata::read_meta(&dest) {
                let (cmp, ok) = compare_versions_safe(&meta.version, &opts.version);
                if ok && cmp == std::cmp::Ordering::Greater && !opts.force {
                    return Ok((
                        ActionKind::Skipped,
                        format!(
                            "skill {:?} has newer version {:?} (> {:?}); use --force to downgrade",
                            skill.dir, meta.version, opts.version
                        ),
                    ));
                }
            }
        }
    }

    let label = match opts.mode {
        CopyMode::Install => ActionKind::Installed,
        CopyMode::Update => ActionKind::Updated,
        CopyMode::Reinstall => ActionKind::Reinstalled,
    };

    if opts.dry_run {
        let label_str = match label {
            ActionKind::Installed => "installed",
            ActionKind::Updated => "updated",
            ActionKind::Reinstalled => "reinstalled",
            _ => unreachable!(),
        };
        return Ok((
            label,
            format!("[dry-run] would {} skill {:?}", label_str, skill.dir),
        ));
    }

    // Backup existing destination before copy.
    let dest_exists = dest.exists();
    let backup = if dest_exists {
        let rand_bytes: [u8; 4] = rand::random();
        let suffix = hex::encode(&rand_bytes);
        let backup_path = dest.with_file_name(format!(
            "{}.{}.bak",
            dest.file_name().unwrap().to_str().unwrap(),
            suffix
        ));
        std::fs::rename(&dest, &backup_path).map_err(|e| CopyError::Copy {
            skill: skill.dir.clone(),
            source: e,
        })?;
        Some(backup_path)
    } else {
        None
    };

    // Perform the actual file copy.
    let copy_result = copy_embedded_dir(src, &skill.dir, &dest);

    if let Err(copy_err) = copy_result {
        // Rollback: remove partial copy and restore backup.
        let _ = std::fs::remove_dir_all(&dest);
        if let Some(ref bak) = backup {
            let _ = std::fs::rename(bak, &dest);
        }
        return Err(CopyError::Copy {
            skill: skill.dir.clone(),
            source: copy_err,
        });
    }

    // Write metadata.
    let meta = SkillMeta {
        installed_by: opts.name.clone(),
        version: opts.version.clone(),
        installed_at: Utc::now(),
    };
    if let Err(e) = metadata::write_meta(&dest, &meta) {
        // Rollback on metadata write failure.
        let _ = std::fs::remove_dir_all(&dest);
        if let Some(ref bak) = backup {
            let _ = std::fs::rename(bak, &dest);
        }
        return Err(CopyError::Other(format!(
            "writing metadata for {:?}: {e}",
            skill.dir
        )));
    }

    // Success — remove backup.
    if let Some(ref bak) = backup {
        let _ = std::fs::remove_dir_all(bak);
    }

    Ok((label, String::new()))
}

/// Recursively copy an embedded directory to disk.
fn copy_embedded_dir(src: &Dir<'_>, skill_dir: &str, dest: &Path) -> Result<(), std::io::Error> {
    let sub = src
        .get_dir(skill_dir)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "skill dir not found"))?;

    copy_dir_recursive(sub, dest)
}

fn copy_dir_recursive(dir: &Dir<'_>, dest: &Path) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(dest)?;

    for file in dir.files() {
        let rel = file.path().strip_prefix(dir.path()).unwrap_or(file.path());
        let dest_file = dest.join(rel);
        if let Some(parent) = dest_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&dest_file, file.contents())?;
    }

    for sub in dir.dirs() {
        let rel = sub.path().strip_prefix(dir.path()).unwrap_or(sub.path());
        let dest_sub = dest.join(rel);
        copy_dir_recursive(sub, &dest_sub)?;
    }

    Ok(())
}

// hex encoding helper (avoid pulling in a full hex crate)
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_versions_safe_semver() {
        let (cmp, ok) = compare_versions_safe("v1.0.0", "v1.1.0");
        assert!(ok);
        assert_eq!(cmp, std::cmp::Ordering::Less);

        let (cmp, ok) = compare_versions_safe("1.2.0", "1.1.0");
        assert!(ok);
        assert_eq!(cmp, std::cmp::Ordering::Greater);

        let (cmp, ok) = compare_versions_safe("v1.0.0", "v1.0.0");
        assert!(ok);
        assert_eq!(cmp, std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_compare_versions_safe_invalid() {
        let (_, ok) = compare_versions_safe("abc", "v1.0.0");
        assert!(!ok);
    }

    #[test]
    fn test_ensure_v_prefix() {
        assert_eq!(ensure_v_prefix("1.0.0"), "v1.0.0");
        assert_eq!(ensure_v_prefix("v1.0.0"), "v1.0.0");
    }
}
