mod cli;
mod daemon;

use crate::cli::{Cli, Commands};
use clap::Parser;
use std::process;

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Start => daemon::start_daemon(),
        Commands::Status => daemon::check_status(),
        Commands::Stop => daemon::stop_daemon(),
    }
}
