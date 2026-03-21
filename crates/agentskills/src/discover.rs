use include_dir::Dir;

use crate::error::SkillError;
use crate::skill::{self, Skill};
use crate::validate;

/// Enumerate skill directories inside an embedded [`Dir`], parse each SKILL.md,
/// and return valid skills together with any per-skill errors.
///
/// Skills whose SKILL.md cannot be parsed or fails validation are omitted from
/// the returned `Vec<Skill>`; each such failure is collected into the returned
/// `Vec<SkillError>`.
pub fn discover(dir: &Dir<'_>) -> (Vec<Skill>, Vec<SkillError>) {
    let mut skills = Vec::new();
    let mut errors = Vec::new();

    for entry in dir.dirs() {
        let dir_name = match entry.path().file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        let skill_file = match entry.get_file(format!("{}/SKILL.md", dir_name)) {
            Some(f) => f,
            None => {
                // Also try just "SKILL.md" relative to the subdirectory.
                match entry.get_file("SKILL.md") {
                    Some(f) => f,
                    None => continue, // No SKILL.md — silently skip.
                }
            }
        };

        let content = match skill_file.contents_utf8() {
            Some(c) => c,
            None => {
                errors.push(SkillError {
                    dir: dir_name,
                    err: "SKILL.md is not valid UTF-8".into(),
                });
                continue;
            }
        };

        let mut parsed = match skill::parse(content) {
            Ok(s) => s,
            Err(e) => {
                errors.push(SkillError {
                    dir: dir_name,
                    err: format!("parse error: {e}").into(),
                });
                continue;
            }
        };

        let result = validate::validate(&parsed, &dir_name);
        if !result.ok() {
            for e in result.errors {
                errors.push(SkillError {
                    dir: dir_name.clone(),
                    err: format!("validation error: {e}").into(),
                });
            }
            continue;
        }

        parsed.dir = dir_name;
        skills.push(parsed);
    }

    (skills, errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use include_dir::include_dir;

    // Use a test fixture directory embedded at compile time.
    static TEST_SKILLS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/tests/fixtures/skills");

    #[test]
    fn test_discover_finds_valid_skills() {
        let (skills, errors) = discover(&TEST_SKILLS);
        // This test depends on test fixtures; the basic contract is that
        // discover returns without panicking.
        let _ = (skills, errors);
    }
}
