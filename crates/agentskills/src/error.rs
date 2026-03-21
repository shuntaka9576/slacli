use std::fmt;

/// Error associated with a specific skill directory.
/// Returned by [`discover`](crate::discover) for per-skill failures.
#[derive(Debug)]
pub struct SkillError {
    pub dir: String,
    pub err: Box<dyn std::error::Error + Send + Sync>,
}

impl fmt::Display for SkillError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.dir, self.err)
    }
}

impl std::error::Error for SkillError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.err.as_ref())
    }
}

/// Errors that can occur when parsing a SKILL.md file.
#[derive(Debug, thiserror::Error)]
pub enum SkillParseError {
    #[error("SKILL.md: missing frontmatter opening delimiter")]
    MissingOpenDelimiter,
    #[error("SKILL.md: missing frontmatter closing delimiter")]
    MissingCloseDelimiter,
    #[error("SKILL.md: YAML parse error: {0}")]
    YamlError(#[from] serde_yml::Error),
}
