use std::io::Write;

use include_dir::Dir;

use crate::discover;

pub fn execute(
    skills_dir: &Dir<'_>,
    out: &mut dyn Write,
    err_w: &mut dyn Write,
) -> Result<(), Box<dyn std::error::Error>> {
    let (skills, errors) = discover::discover(skills_dir);

    for e in &errors {
        writeln!(err_w, "warning: {e}")?;
    }

    if skills.is_empty() {
        writeln!(out, "no skills found")?;
        return Ok(());
    }

    for sk in &skills {
        if !sk.description.is_empty() {
            writeln!(out, "{:<30} {}", sk.dir, sk.description)?;
        } else {
            writeln!(out, "{}", sk.dir)?;
        }
    }
    Ok(())
}
