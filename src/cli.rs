//! Interface de linha de comando do WOLRAM baseada em clap.
//!
//! Define a struct [`Cli`] com subcomandos [`Command`] (run, demo, status)
//! e flags globais (--model, --max-retries, --verbose).

use clap::{Parser, Subcommand, ValueEnum};

/// WOLRAM — Orquestrador de desenvolvimento IA de nível empresarial.
#[derive(Debug, Parser)]
#[command(name = "wolram", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Nível de modelo a usar nesta sessão.
    #[arg(long, global = true)]
    pub model: Option<ModelArg>,

    /// Número máximo de retentativas em caso de falha.
    #[arg(long, global = true)]
    pub max_retries: Option<u32>,

    /// Habilita saída detalhada (verbose).
    #[arg(long, short, global = true, default_value_t = false)]
    pub verbose: bool,
}

/// Argumento de modelo aceito pela CLI, mapeado para [`ModelTier`](crate::state_machine::ModelTier) internamente.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ModelArg {
    /// Modelo rápido e econômico para tarefas simples.
    Haiku,
    /// Modelo equilibrado para tarefas de complexidade média.
    Sonnet,
    /// Modelo mais capaz para tarefas complexas.
    Opus,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Executa um job com a descrição fornecida.
    Run {
        /// Descrição do job (o que construir/corrigir/refatorar).
        description: Option<String>,

        /// Caminho para um arquivo JSON ou TOML contendo definições de job.
        #[arg(long)]
        file: Option<String>,
    },

    /// Mostra o status atual do orquestrador.
    Status,

    /// Generates a TODO list from a natural language prompt.
    Todo {
        /// Natural language description of what needs to be done.
        prompt: String,
    },

    /// Executa a demonstração embutida da máquina de estados.
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
    fn cli_parses_todo_subcommand() {
        let cli = Cli::parse_from(["wolram", "todo", "implement auth and add tests"]);
        match cli.command {
            Command::Todo { prompt } => {
                assert_eq!(prompt, "implement auth and add tests");
            }
            _ => panic!("expected Todo command"),
        }
    }

    #[test]
    fn cli_verify() {
        Cli::command().debug_assert();
    }
}
