use std::collections::HashMap;

use serde::Deserialize;

use crate::error::SkillParseError;

/// A parsed agentskill from a SKILL.md file.
#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub license: String,
    pub compatibility: Vec<String>,
    pub metadata: HashMap<String, serde_yml::Value>,
    pub allowed_tools: Vec<String>,
    pub body: String,
    /// Directory name of the skill (set by [`discover`](crate::discover)).
    pub dir: String,
}

/// Raw YAML fields parsed from the SKILL.md frontmatter.
#[derive(Debug, Deserialize)]
struct Frontmatter {
    #[serde(default)]
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    license: String,
    #[serde(default)]
    compatibility: Vec<String>,
    #[serde(default)]
    metadata: HashMap<String, serde_yml::Value>,
    #[serde(default)]
    allowed_tools: Vec<String>,
}

/// Parse a SKILL.md file content into a [`Skill`].
///
/// The content must have YAML frontmatter delimited by `---` lines.
pub fn parse(content: &str) -> Result<Skill, SkillParseError> {
    let (yaml_str, body) = split_frontmatter(content)?;

    let fm: Frontmatter = serde_yml::from_str(yaml_str)?;

    Ok(Skill {
        name: fm.name,
        description: fm.description,
        license: fm.license,
        compatibility: fm.compatibility,
        metadata: fm.metadata,
        allowed_tools: fm.allowed_tools,
        body: body.to_string(),
        dir: String::new(),
    })
}

/// Split SKILL.md content into YAML frontmatter and body.
fn split_frontmatter(content: &str) -> Result<(&str, &str), SkillParseError> {
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() || lines[0].trim() != "---" {
        return Err(SkillParseError::MissingOpenDelimiter);
    }

    let closing_idx = lines[1..]
        .iter()
        .position(|line| line.trim() == "---")
        .map(|i| i + 1);

    let closing_idx = match closing_idx {
        Some(idx) => idx,
        None => return Err(SkillParseError::MissingCloseDelimiter),
    };

    // Calculate byte offsets for zero-copy slicing.
    // Skip the opening "---\n".
    let yaml_start = content.find('\n').map(|i| i + 1).unwrap_or(content.len());

    // Find the closing "---" line start position.
    let mut pos = 0;
    for (i, line) in content.lines().enumerate() {
        if i == closing_idx {
            break;
        }
        pos += line.len() + 1; // +1 for newline
    }
    let yaml_end = pos.saturating_sub(1); // exclude trailing newline

    let yaml_str = &content[yaml_start..yaml_end];

    // Body starts after closing "---\n".
    let body_start = pos + lines[closing_idx].len() + 1;
    let body = if body_start < content.len() {
        content[body_start..].trim_start_matches('\n')
    } else {
        ""
    };

    Ok((yaml_str, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_skill() {
        let content = r#"---
name: my-skill
description: A test skill
license: MIT
compatibility:
  - claude
  - codex
allowed_tools:
  - Bash
  - Read
---
# My Skill

This is the body.
"#;
        let skill = parse(content).unwrap();
        assert_eq!(skill.name, "my-skill");
        assert_eq!(skill.description, "A test skill");
        assert_eq!(skill.license, "MIT");
        assert_eq!(skill.compatibility, vec!["claude", "codex"]);
        assert_eq!(skill.allowed_tools, vec!["Bash", "Read"]);
        assert!(skill.body.starts_with("# My Skill"));
    }

    #[test]
    fn test_parse_minimal_frontmatter() {
        let content = "---\nname: minimal\ndescription: desc\n---\n";
        let skill = parse(content).unwrap();
        assert_eq!(skill.name, "minimal");
        assert_eq!(skill.description, "desc");
        assert!(skill.license.is_empty());
        assert!(skill.compatibility.is_empty());
    }

    #[test]
    fn test_parse_missing_open_delimiter() {
        let content = "name: bad\n---\n";
        let err = parse(content).unwrap_err();
        assert!(matches!(err, SkillParseError::MissingOpenDelimiter));
    }

    #[test]
    fn test_parse_missing_close_delimiter() {
        let content = "---\nname: bad\n";
        let err = parse(content).unwrap_err();
        assert!(matches!(err, SkillParseError::MissingCloseDelimiter));
    }

    #[test]
    fn test_parse_with_metadata() {
        let content =
            "---\nname: m\ndescription: d\nmetadata:\n  author: test\n  version: \"1.0\"\n---\n";
        let skill = parse(content).unwrap();
        assert!(skill.metadata.contains_key("author"));
    }
}
