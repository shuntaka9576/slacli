use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const META_FILENAME: &str = ".agentskills.json";

/// Metadata written alongside an installed skill.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillMeta {
    pub installed_by: String,
    pub version: String,
    pub installed_at: DateTime<Utc>,
}

/// Read `.agentskills.json` from the given skill directory.
pub fn read_meta(dir: &Path) -> Result<SkillMeta, MetaError> {
    let path = dir.join(META_FILENAME);
    let data = std::fs::read_to_string(&path).map_err(|e| MetaError::Io(path.clone(), e))?;
    let meta: SkillMeta =
        serde_json::from_str(&data).map_err(|e| MetaError::Json(path.clone(), e))?;
    Ok(meta)
}

/// Write metadata as `.agentskills.json` into `dir`.
pub fn write_meta(dir: &Path, meta: &SkillMeta) -> Result<(), MetaError> {
    let path = dir.join(META_FILENAME);
    let data = serde_json::to_string(meta).map_err(|e| MetaError::Json(path.clone(), e))?;
    std::fs::write(&path, data).map_err(|e| MetaError::Io(path, e))?;
    Ok(())
}

/// Check whether a `.agentskills.json` file exists in `dir`.
pub fn is_managed(dir: &Path) -> bool {
    dir.join(META_FILENAME).exists()
}

/// Errors from metadata operations.
#[derive(Debug, thiserror::Error)]
pub enum MetaError {
    #[error("IO error for {0}: {1}")]
    Io(std::path::PathBuf, #[source] std::io::Error),
    #[error("JSON error for {0}: {1}")]
    Json(std::path::PathBuf, #[source] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tempfile::TempDir;

    #[test]
    fn test_write_and_read_meta() {
        let tmp = TempDir::new().unwrap();
        let meta = SkillMeta {
            installed_by: "test-tool".to_string(),
            version: "v1.0.0".to_string(),
            installed_at: Utc::now(),
        };
        write_meta(tmp.path(), &meta).unwrap();
        let read = read_meta(tmp.path()).unwrap();
        assert_eq!(read.installed_by, "test-tool");
        assert_eq!(read.version, "v1.0.0");
    }

    #[test]
    fn test_is_managed() {
        let tmp = TempDir::new().unwrap();
        assert!(!is_managed(tmp.path()));

        let meta = SkillMeta {
            installed_by: "t".to_string(),
            version: "v0.1.0".to_string(),
            installed_at: Utc::now(),
        };
        write_meta(tmp.path(), &meta).unwrap();
        assert!(is_managed(tmp.path()));
    }

    #[test]
    fn test_json_field_names_are_camel_case() {
        let meta = SkillMeta {
            installed_by: "tool".to_string(),
            version: "v1.0.0".to_string(),
            installed_at: Utc::now(),
        };
        let json = serde_json::to_string(&meta).unwrap();
        assert!(json.contains("installedBy"));
        assert!(json.contains("installedAt"));
        assert!(!json.contains("installed_by"));
    }
}
