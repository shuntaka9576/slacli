pub mod commands;
pub mod copy;
pub mod discover;
pub mod error;
pub mod metadata;
pub mod resolve;
pub mod skill;
pub mod validate;

use std::io::Write;

use clap::Parser;
use include_dir::Dir;

use commands::{SkillsCli, SkillsCommand};

/// Main entry point for the skills subcommand.
///
/// Create one with [`Smith::new`] and call [`Smith::run`] to dispatch
/// to the appropriate subcommand.
pub struct Smith {
    name: String,
    version: String,
    skills_dir: &'static Dir<'static>,
    pub out: Box<dyn Write>,
    pub err_w: Box<dyn Write>,
}

/// Errors from [`Smith`] operations.
#[derive(Debug, thiserror::Error)]
pub enum SmithError {
    #[error("invalid version {0:?}")]
    InvalidVersion(String),
    #[error("skill filesystem cannot be empty")]
    EmptySkillFS,
    #[error("{0}")]
    Other(String),
}

impl Smith {
    /// Create a new Smith with the given tool name, version, and embedded skill directory.
    ///
    /// - Version is validated as semver (a `v` prefix is auto-prepended for validation if missing).
    /// - If the root of `skills_dir` contains exactly one subdirectory named `"skills"`,
    ///   that directory is used as the skill root.
    pub fn new(
        name: &str,
        version: &str,
        skills_dir: &'static Dir<'static>,
    ) -> Result<Self, SmithError> {
        // Validate version.
        let v = version.strip_prefix('v').unwrap_or(version);
        if semver::Version::parse(v).is_err() {
            return Err(SmithError::InvalidVersion(version.to_string()));
        }

        // Auto-detect skills/ prefix.
        // Note: include_dir::Dir doesn't support Sub like Go's fs.Sub,
        // so we record whether to strip and handle it in discover.
        // For now, we pass the dir as-is since discover handles subdirectories.

        Ok(Smith {
            name: name.to_string(),
            version: version.to_string(),
            skills_dir,
            out: Box::new(std::io::stdout()),
            err_w: Box::new(std::io::stderr()),
        })
    }

    /// Name of the hosting CLI tool.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Version string as provided to [`new`](Smith::new).
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Parse args and dispatch to the matching subcommand.
    pub fn run(&mut self, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
        // Prepend a dummy program name for clap.
        let mut full_args = vec!["skills".to_string()];
        full_args.extend_from_slice(args);

        let cli = match SkillsCli::try_parse_from(&full_args) {
            Ok(c) => c,
            Err(e) => {
                // Print help/error to err_w.
                write!(self.err_w, "{e}")?;
                return Ok(());
            }
        };

        // Detect skills/ prefix auto-stripping.
        let effective_dir = auto_detect_skills_dir(self.skills_dir);

        match cli.command {
            SkillsCommand::List => {
                commands::list::execute(effective_dir, &mut self.out, &mut self.err_w)?;
            }
            SkillsCommand::Install(ref a) => {
                commands::install::execute(
                    effective_dir,
                    &self.name,
                    &self.version,
                    a,
                    &mut self.out,
                    &mut self.err_w,
                )?;
            }
            SkillsCommand::Update(ref a) => {
                commands::update::execute(
                    effective_dir,
                    &self.name,
                    &self.version,
                    a,
                    &mut self.out,
                    &mut self.err_w,
                )?;
            }
            SkillsCommand::Reinstall(ref a) => {
                commands::reinstall::execute(
                    effective_dir,
                    &self.name,
                    &self.version,
                    a,
                    &mut self.out,
                    &mut self.err_w,
                )?;
            }
            SkillsCommand::Uninstall(ref a) => {
                commands::uninstall::execute(
                    effective_dir,
                    &self.name,
                    a,
                    &mut self.out,
                    &mut self.err_w,
                )?;
            }
            SkillsCommand::Status(ref a) => {
                commands::status::execute(
                    effective_dir,
                    &self.version,
                    a,
                    &mut self.out,
                    &mut self.err_w,
                )?;
            }
        }

        Ok(())
    }
}

/// If the root contains exactly one subdirectory named `"skills"`, return it.
/// Otherwise return the original directory.
fn auto_detect_skills_dir(dir: &'static Dir<'static>) -> &'static Dir<'static> {
    let subdirs: Vec<_> = dir.dirs().collect();
    if subdirs.len() == 1 {
        if let Some(name) = subdirs[0].path().file_name().and_then(|n| n.to_str()) {
            if name == "skills" {
                return subdirs[0];
            }
        }
    }
    dir
}
