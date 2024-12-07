mod cli;
mod config;
mod daemon;
mod protocol;
mod protocol_v1;

use crate::cli::{Cli, Commands, ConfigCommands};
use clap::Parser;
use std::process;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match &cli.command {
        Commands::Start => daemon::start_daemon(crate::config::Config::load()?),
        Commands::Status => daemon::check_status(),
        Commands::Stop => daemon::stop_daemon(),
        Commands::Send { command } => Ok(if let Err(e) = daemon::send_command(command) {
            eprintln!("Error sending ping: {}", e);
            process::exit(1);
        }),
        Commands::Config { command } => Ok(if let Err(e) = handle_config(command) {
            eprintln!("Error handling config: {}", e);
            process::exit(1);
        }),
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
