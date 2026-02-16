mod anthropic;
mod cli;
mod config;
mod git;
mod orchestrator;
mod router;
mod state_machine;

use anyhow::{bail, Result};
use clap::Parser;

use cli::{Cli, Command, ModelArg};
use config::WolramConfig;
use orchestrator::JobOrchestrator;
use state_machine::{
    AuditRecord, Job, JobOutcome, ModelTier, RetryConfig, StateMachine, Transition,
};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = WolramConfig::load()?;

    // Resolve effective values: CLI overrides > config > defaults.
    let max_retries = cli.max_retries.unwrap_or(config.max_retries);
    let model_tier = cli
        .model
        .map(model_arg_to_tier)
        .unwrap_or_else(|| parse_model_tier(&config.default_model_tier));
    let verbose = cli.verbose;

    match cli.command {
        Command::Run { description, file } => {
            let mut job = if let Some(path) = file {
                let contents = std::fs::read_to_string(&path)
                    .map_err(|e| anyhow::anyhow!("Failed to read {path}: {e}"))?;
                serde_json::from_str::<Job>(&contents)
                    .map_err(|e| anyhow::anyhow!("Failed to parse job JSON from {path}: {e}"))?
            } else if let Some(desc) = description {
                Job::new(
                    desc,
                    RetryConfig {
                        max_retries,
                        base_delay_ms: config.base_delay_ms,
                    },
                )
            } else {
                bail!("Provide a job description or --file");
            };

            let client = if config.api_key.is_empty() {
                None
            } else {
                Some(anthropic::AnthropicClient::new(config.api_key.clone()))
            };
            let orch = JobOrchestrator::new(client, false);

            if verbose {
                eprintln!("Starting job: {} ({})", job.description, job.id);
                eprintln!("Model: {model_tier}, Max retries: {max_retries}");
            }

            let record = orch.run_job(&mut job).await?;

            if verbose {
                eprintln!("State transitions: {:?}", record.state_transitions);
            }

            println!("{}", serde_json::to_string_pretty(&record)?);
        }

        Command::Status => {
            println!("Status tracking not yet implemented");
        }

        Command::Demo => {
            run_demo();
        }
    }

    Ok(())
}

fn model_arg_to_tier(arg: ModelArg) -> ModelTier {
    match arg {
        ModelArg::Haiku => ModelTier::Haiku,
        ModelArg::Sonnet => ModelTier::Sonnet,
        ModelArg::Opus => ModelTier::Opus,
    }
}

fn parse_model_tier(s: &str) -> ModelTier {
    match s.to_lowercase().as_str() {
        "haiku" => ModelTier::Haiku,
        "opus" => ModelTier::Opus,
        _ => ModelTier::Sonnet,
    }
}

fn run_demo() {
    let mut job = Job::new(
        "Example: implement hero section layout".to_string(),
        RetryConfig::default(),
    );

    println!("WOLRAM — Job State Machine Demo");
    println!("Job: {} ({})", job.description, job.id);
    println!("State: {}\n", job.state);

    // INIT → DEFINE_AGENT
    StateMachine::next(&mut job, JobOutcome::Success);
    println!("  → Transitioned to {}", job.state);

    // Simulate agent assignment during DEFINE_AGENT phase.
    job.assign_agent("code_generation".to_string(), ModelTier::Sonnet);
    println!(
        "  ✦ Assigned agent: skill={}, model={}, est. cost=${:.3}",
        job.agent.as_ref().unwrap().skill,
        job.agent.as_ref().unwrap().model,
        job.estimated_cost_usd(),
    );

    // Walk through remaining states.
    let outcomes = [JobOutcome::Success, JobOutcome::Success];

    for outcome in outcomes {
        let transition = StateMachine::next(&mut job, outcome);
        match &transition {
            Transition::Next(state) => println!("  → Transitioned to {state}"),
            Transition::Retry { state, reason } => {
                println!(
                    "  ↻ Retrying {state} (attempt {}/{}): {reason}",
                    job.retry_count, job.retry_config.max_retries
                );
            }
            Transition::Complete(outcome) => println!("  ■ Completed: {outcome:?}"),
        }
    }

    let record = AuditRecord::from_job(&job);
    println!("\nAudit Record:");
    println!("{}", serde_json::to_string_pretty(&record).unwrap());
}
