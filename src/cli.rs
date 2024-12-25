use clap::{Parser, Subcommand};
use std::path::PathBuf;

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

    /// Restart the daemon
    Restart,

    /// Configure the daemon
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Get daemon version
    Version,

    /// List all peers
    Peers,

    /// List active sessions
    Sessions,

    /// Get daemon info
    Info,

    /// Refresh the daemon's peers
    Refresh,

    /// Send a file to a peer
    File {
        /// Peer fingerprint
        peer: String,

        /// Path to the file to send
        #[arg(value_parser = clap::value_parser!(PathBuf))]
        path: PathBuf,
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
