use std::io::Write;

use include_dir::Dir;

use super::CommonArgs;
use crate::discover;
use crate::metadata;
use crate::resolve;

pub fn execute(
    skills_dir: &Dir<'_>,
    name: &str,
    args: &CommonArgs,
    out: &mut dyn Write,
    err_w: &mut dyn Write,
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = resolve::install_dir(args.prefix.as_deref(), Some(&args.scope))?;

    let (skills, errors) = discover::discover(skills_dir);

    for e in &errors {
        writeln!(err_w, "warning: {e}")?;
    }

    for skill in &skills {
        let dest = dir.join(&skill.dir);
        if !metadata::is_managed(&dest) {
            writeln!(
                out,
                "skipped:     {} — not managed by {}",
                skill.dir, name
            )?;
            continue;
        }

        if args.dry_run {
            writeln!(out, "uninstalled (dry-run): {}", skill.dir)?;
            continue;
        }

        std::fs::remove_dir_all(&dest).map_err(|e| format!("uninstalling {:?}: {e}", skill.dir))?;
        writeln!(out, "uninstalled: {}", skill.dir)?;
    }

    if args.dry_run {
        writeln!(out, "[dry-run] no changes were made")?;
    }
    Ok(())
}
