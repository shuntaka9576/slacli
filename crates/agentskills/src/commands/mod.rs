pub mod install;
pub mod list;
pub mod reinstall;
pub mod status;
pub mod uninstall;
pub mod update;

use clap::{Args, Parser, Subcommand};

/// CLI definition for the `skills` subcommand.
#[derive(Parser, Debug)]
#[command(name = "skills")]
pub struct SkillsCli {
    #[command(subcommand)]
    pub command: SkillsCommand,
}

#[derive(Subcommand, Debug)]
pub enum SkillsCommand {
    /// List embedded skills
    List,
    /// Install skills (skip existing)
    Install(CommonArgs),
    /// Update skills when version differs
    Update(CommonArgs),
    /// Reinstall all managed skills
    Reinstall(CommonArgs),
    /// Uninstall managed skills
    Uninstall(CommonArgs),
    /// Show install status and version diff
    Status(CommonArgs),
}

/// Shared options for install/update/reinstall/uninstall/status subcommands.
#[derive(Args, Debug, Clone)]
pub struct CommonArgs {
    /// Print what would happen without making changes
    #[arg(long)]
    pub dry_run: bool,

    /// Skill installation directory (overrides --scope)
    #[arg(long)]
    pub prefix: Option<String>,

    /// Target scope: user (~/.agents/skills, default) or repo
    #[arg(long, default_value = "user")]
    pub scope: String,

    /// Overwrite unmanaged skills or force downgrade
    #[arg(long)]
    pub force: bool,
}
