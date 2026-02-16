mod cli;
mod config;
mod error;

use clap::Parser;
use cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Init { prompt } => {
            println!("Initializing project with prompt: {prompt}");
        }
        Command::Run => {
            println!("Running pending jobs...");
        }
        Command::Status => {
            println!("Showing job status...");
        }
    }
}
