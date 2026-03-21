use std::io::Write;

use include_dir::Dir;

use super::CommonArgs;
use crate::discover;
use crate::metadata;
use crate::resolve;

pub fn execute(
    skills_dir: &Dir<'_>,
    version: &str,
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
            writeln!(out, "{:<30} not installed", skill.dir)?;
            continue;
        }

        match metadata::read_meta(&dest) {
            Ok(meta) => {
                let up_to_date = is_up_to_date(&meta.version, version);
                if up_to_date {
                    writeln!(
                        out,
                        "{:<30} installed {} (up to date)",
                        skill.dir, meta.version
                    )?;
                } else {
                    writeln!(
                        out,
                        "{:<30} installed {} → available {}",
                        skill.dir, meta.version, version
                    )?;
                }
            }
            Err(e) => {
                writeln!(
                    out,
                    "{:<30} installed (metadata unreadable: {e})",
                    skill.dir
                )?;
            }
        }
    }
    Ok(())
}

fn is_up_to_date(installed: &str, current: &str) -> bool {
    let i_str = installed.strip_prefix('v').unwrap_or(installed);
    let c_str = current.strip_prefix('v').unwrap_or(current);
    let parsed_i = semver::Version::parse(i_str);
    let parsed_c = semver::Version::parse(c_str);

    match (parsed_i, parsed_c) {
        (Ok(i), Ok(c)) => i >= c,
        _ => installed == current,
    }
}
