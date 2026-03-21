use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "slacli", about = "Slack CLI", disable_version_flag = true)]
pub struct Cli {
    /// Print version
    #[arg(long, short = 'V')]
    pub version: bool,
    /// Profile name (overrides default_profile in config.toml)
    #[arg(long, global = true)]
    pub profile: Option<String>,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize tokens interactively
    Init,
    /// Show configuration
    Config {
        /// Show current configuration as JSON
        #[arg(long)]
        see: bool,
    },
    /// Chat operations
    Chat {
        #[command(subcommand)]
        command: ChatCommands,
    },
    /// Profile operations
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },
    /// View local logs of previously executed commands
    Logs {
        /// Log type to display (e.g. chat-send)
        #[arg(long = "type")]
        log_type: String,
        /// Remove all logs of the specified type
        #[arg(long)]
        purge: bool,
    },
}

#[derive(Subcommand)]
pub enum ChatCommands {
    /// Send a message to a channel
    Send {
        /// Channel ID, name, or alias (defined in config.toml)
        #[arg(short, long)]
        channel: String,
        /// Message text
        #[arg(short, long)]
        text: String,
        /// Thread timestamp to reply to (creates a threaded reply)
        #[arg(short = 'T', long)]
        thread_ts: Option<String>,
    },
    /// Delete a message from a channel
    Delete {
        /// Channel ID or alias (defined in config.toml)
        #[arg(short, long)]
        channel: String,
        /// Message timestamp (ts)
        #[arg(short = 's', long)]
        timestamp: String,
    },
}

#[derive(Subcommand)]
pub enum ProfileCommands {
    /// Edit your profile (status, name, etc.)
    Edit {
        /// Profile field to set (key=value, repeatable)
        #[arg(short = 's', long = "set", value_parser = parse_key_value)]
        fields: Vec<(String, String)>,
    },
}

fn parse_key_value(s: &str) -> Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=VALUE: no '=' found in '{s}'"))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}
