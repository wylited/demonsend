use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the daemon
    Start,
    /// Check if daemon is running
    Status,
    /// Stop the daemon
    Stop,
    /// Ping the daemon
    Ping,
    /// Configure the daemon
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Set configuration values
    Set {
        #[arg(long)]
        download_dir: Option<String>,
    },
    /// Show current configuration
    Show,
    /// Initialize configuration interactively
    Init,
}
