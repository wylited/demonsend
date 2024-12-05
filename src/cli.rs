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
}
