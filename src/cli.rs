use clap::{Parser, Subcommand, ValueEnum};

/// WOLRAM — Enterprise-grade AI development orchestrator.
#[derive(Debug, Parser)]
#[command(name = "wolram", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Model tier to use for this session.
    #[arg(long, global = true)]
    pub model: Option<ModelArg>,

    /// Maximum number of retries on failure.
    #[arg(long, global = true)]
    pub max_retries: Option<u32>,

    /// Enable verbose output.
    #[arg(long, short, global = true, default_value_t = false)]
    pub verbose: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ModelArg {
    Haiku,
    Sonnet,
    Opus,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run a job with the given description.
    Run {
        /// Job description (what to build/fix/refactor).
        description: Option<String>,

        /// Path to a JSON or TOML file containing job definitions.
        #[arg(long)]
        file: Option<String>,
    },

    /// Show current orchestrator status.
    Status,

    /// Run the built-in state machine demo.
    Demo,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_parses_run_subcommand() {
        let cli = Cli::parse_from(["wolram", "run", "implement hero section"]);
        match cli.command {
            Command::Run { description, file } => {
                assert_eq!(description.unwrap(), "implement hero section");
                assert!(file.is_none());
            }
            _ => panic!("expected Run command"),
        }
    }

    #[test]
    fn cli_parses_global_flags() {
        let cli = Cli::parse_from([
            "wolram",
            "--model",
            "opus",
            "--max-retries",
            "5",
            "--verbose",
            "demo",
        ]);
        assert!(cli.verbose);
        assert!(matches!(cli.model, Some(ModelArg::Opus)));
        assert_eq!(cli.max_retries, Some(5));
    }

    #[test]
    fn cli_verify() {
        Cli::command().debug_assert();
    }
}
