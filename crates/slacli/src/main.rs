mod cli;
mod client;
mod commands;
mod config;
mod error;
mod state;

use agentskills::Smith;
use clap::Parser;
use include_dir::{include_dir, Dir};

use cli::{ChatCommands, Cli, Commands, ProfileCommands};

const APP_VERSION: &str = concat!(
    env!("CARGO_PKG_NAME"),
    " version ",
    env!("CARGO_PKG_VERSION"),
    " (rev:",
    env!("GIT_HASH"),
    ")"
);

static SKILLS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/skills");

fn main() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("failed to install rustls crypto provider");

    // Check if the first argument is "skills" to delegate to agentskills.
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(|s| s.as_str()) == Some("skills") {
        let mut smith = Smith::new("slacli", env!("CARGO_PKG_VERSION"), &SKILLS_DIR)
            .expect("failed to initialize smith");
        if let Err(e) = smith.run(&args[1..]) {
            eprintln!("{e}");
            std::process::exit(1);
        }
        return;
    }

    let cli = Cli::parse();

    if cli.version {
        println!("{APP_VERSION}");
        return;
    }

    let Some(command) = cli.command else {
        use clap::CommandFactory;
        Cli::command().print_help().unwrap();
        std::process::exit(1);
    };

    let profile = cli.profile.as_deref();

    let result = match command {
        Commands::Init => commands::init::execute(),
        Commands::Config { see } => {
            if see {
                commands::config_cmd::see(profile)
            } else {
                commands::config_cmd::open_editor()
            }
        }
        Commands::Chat { command } => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                match command {
                    ChatCommands::Send {
                        channel,
                        text,
                        thread_ts,
                    } => commands::chat::send(&channel, &text, thread_ts.as_deref(), profile).await,
                    ChatCommands::Delete { channel, timestamp } => {
                        commands::chat::delete(&channel, &timestamp, profile).await
                    }
                }
            })
        }
        Commands::Profile { command } => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                match command {
                    ProfileCommands::Edit { fields } => {
                        commands::profile::edit(fields, profile).await
                    }
                }
            })
        }
        Commands::Logs { log_type, purge } => commands::logs::run(&log_type, purge),
    };

    if let Err(e) = result {
        e.exit();
    }
}
