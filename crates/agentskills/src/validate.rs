use crate::skill::Skill;

/// Result of validating a [`Skill`].
///
/// Warnings do not prevent installation; errors cause the skill to be skipped.
#[derive(Debug, Default)]
pub struct ValidationResult {
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl ValidationResult {
    /// Returns `true` when there are no errors (warnings are allowed).
    pub fn ok(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Perform lenient validation of a skill against its directory name.
pub fn validate(skill: &Skill, dir_name: &str) -> ValidationResult {
    let mut result = ValidationResult::default();

    // description empty/missing → Error (skip skill)
    if skill.description.is_empty() {
        result
            .errors
            .push("description is empty or missing".to_string());
    }

    // name mismatch with directory → Warning (not error)
    if !skill.name.is_empty() && !dir_name.is_empty() && skill.name != dir_name {
        result.warnings.push(format!(
            "skill name {:?} does not match directory name {:?}",
            skill.name, dir_name
        ));
    }

    // name > 64 chars → Warning
    if skill.name.len() > 64 {
        result
            .warnings
            .push(format!("skill name {:?} exceeds 64 characters", skill.name));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_skill(name: &str, description: &str) -> Skill {
        Skill {
            name: name.to_string(),
            description: description.to_string(),
            license: String::new(),
            compatibility: vec![],
            metadata: HashMap::new(),
            allowed_tools: vec![],
            body: String::new(),
            dir: String::new(),
        }
    }

    #[test]
    fn test_valid_skill() {
        let skill = make_skill("my-skill", "A description");
        let result = validate(&skill, "my-skill");
        assert!(result.ok());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_empty_description_is_error() {
        let skill = make_skill("my-skill", "");
        let result = validate(&skill, "my-skill");
        assert!(!result.ok());
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_name_mismatch_is_warning() {
        let skill = make_skill("skill-a", "desc");
        let result = validate(&skill, "skill-b");
        assert!(result.ok());
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_long_name_is_warning() {
        let long_name = "a".repeat(65);
        let skill = make_skill(&long_name, "desc");
        let result = validate(&skill, &long_name);
        assert!(result.ok());
        assert_eq!(result.warnings.len(), 1);
    }
}
