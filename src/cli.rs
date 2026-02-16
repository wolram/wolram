use clap::{Parser, Subcommand};

/// Enterprise-grade orchestration for AI-assisted development
#[derive(Parser)]
#[command(name = "wolram", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize a new project: generate TODO jobs from a prompt
    Init {
        /// Natural language description of what to build
        prompt: String,
    },

    /// Run all pending jobs sequentially through the state machine
    Run,

    /// Show the status of all jobs in the queue
    Status,
}
