use std::io::Write;

use include_dir::Dir;

use super::CommonArgs;
use crate::copy::{copy_skills, ActionKind, CopyMode, CopyOptions};
use crate::resolve;

pub fn execute(
    skills_dir: &Dir<'_>,
    name: &str,
    version: &str,
    args: &CommonArgs,
    out: &mut dyn Write,
    err_w: &mut dyn Write,
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = resolve::install_dir(args.prefix.as_deref(), Some(&args.scope))?;

    let result = copy_skills(
        skills_dir,
        &dir,
        &CopyOptions {
            mode: CopyMode::Reinstall,
            force: args.force,
            dry_run: args.dry_run,
            name: name.to_string(),
            version: version.to_string(),
        },
    )?;

    for a in &result.actions {
        match a.action {
            ActionKind::Reinstalled => {
                if args.dry_run {
                    writeln!(out, "reinstalled (dry-run): {}", a.dir)?;
                } else {
                    writeln!(out, "reinstalled: {}", a.dir)?;
                }
            }
            ActionKind::Warned => {
                writeln!(err_w, "warning:     {} — {}", a.dir, a.message)?;
            }
            _ => {}
        }
    }

    if args.dry_run {
        writeln!(out, "[dry-run] no changes were made")?;
    }
    Ok(())
}
