mod cli;
mod config;
mod daemon;

use crate::cli::{Cli, Commands, ConfigCommands};
use clap::Parser;
use daemon::send_command;
use std::process;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match &cli.command {
        Commands::Start => daemon::start_daemon(crate::config::Config::load()?),
        Commands::Status => daemon::check_status(),
        Commands::Stop => daemon::stop_daemon(),
        Commands::Restart => {
            daemon::stop_daemon()?;
            daemon::start_daemon(crate::config::Config::load()?)
        }
        Commands::Config { command } => Ok(if let Err(e) = handle_config(command) {
            eprintln!("Error handling config: {}", e);
            process::exit(1);
        }),
        Commands::Version => send_command(&"version".to_string()),
        Commands::Peers => send_command(&"peers".to_string()),
        Commands::Sessions => send_command(&"sessions".to_string()),
        Commands::Info => send_command(&"info".to_string()),
        Commands::Refresh => send_command(&"refresh".to_string()),
        Commands::File { peer, path } => send_command(&format!("send {} {}", peer, path.display())),
    }
}

fn handle_config(command: &ConfigCommands) -> anyhow::Result<()> {
    match command {
        ConfigCommands::Show => {
            let config = crate::config::Config::load()?;
            println!("Current configuration:");
            println!("Default Downloads Directory: {:?}", config.download_dir);
        }
        ConfigCommands::Set { download_dir } => {
            let mut config = crate::config::Config::load()?;

            if let Some(download_dir) = download_dir {
                config.download_dir = download_dir.clone();
            }

            config.save()?;
            println!("Configuration updated successfully!");
        }
        ConfigCommands::Init {} => {
            let config = crate::config::Config::initialize_interactive()?;
            config.save()?;
        }
    }
    Ok(())
}
