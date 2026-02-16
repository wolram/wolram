mod anthropic;
mod cli;
mod config;
mod git;
mod orchestrator;
mod router;
mod state_machine;
mod ui;

use anyhow::{bail, Result};
use clap::Parser;

use cli::{Cli, Command, ModelArg};
use config::WolramConfig;
use orchestrator::JobOrchestrator;
use state_machine::{
    AuditRecord, Job, JobOutcome, JobStatus, ModelTier, RetryConfig, State, StateMachine,
    Transition,
};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = WolramConfig::load()?;

    // Resolve effective values: CLI overrides > config > defaults.
    let max_retries = cli.max_retries.unwrap_or(config.max_retries);
    let verbose = cli.verbose;

    match cli.command {
        Command::Run { description, file } => {
            let mut job = if let Some(path) = file {
                let contents = std::fs::read_to_string(&path)
                    .map_err(|e| anyhow::anyhow!("Failed to read {path}: {e}"))?;
                let mut loaded = serde_json::from_str::<Job>(&contents)
                    .map_err(|e| anyhow::anyhow!("Failed to parse job JSON from {path}: {e}"))?;
                // Sanitize: force clean initial state to prevent manipulation
                loaded.state = State::Init;
                loaded.retry_count = 0;
                loaded.state_history.clear();
                loaded.status = JobStatus::Pending;
                loaded
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
            let has_git = git::GitManager::open(std::path::Path::new(".")).is_ok();
            let model_override = cli.model.map(model_arg_to_tier);
            let orch = JobOrchestrator::with_model_override(client, has_git, model_override);

            let progress = ui::JobProgress::start(&job.description);

            if verbose {
                eprintln!("Starting job: {} ({})", job.description, job.id);
                eprintln!("Model override: {:?}, Max retries: {max_retries}", orch.model_override);
            }

            let record = orch.run_job(&mut job).await?;

            progress.complete(&state_machine::JobOutcome::Success);
            progress.print_audit(&record);
        }

        Command::Status => {
            println!("WOLRAM — Status");
            println!();

            // Configuration.
            println!("Configuration:");
            println!("  Default model tier : {}", config.default_model_tier);
            println!("  Max retries        : {max_retries}");
            println!("  Base delay         : {} ms", config.base_delay_ms);
            println!("  Config file        : {}", if std::path::Path::new("wolram.toml").exists() { "wolram.toml (loaded)" } else { "not found (using defaults)" });
            println!();

            // API key status.
            println!("Anthropic API:");
            if config.api_key.is_empty() {
                println!("  API key : not configured (jobs will run in stub mode)");
            } else {
                println!("  API key : configured");
            }
            println!();

            // Git status.
            println!("Git:");
            match git::GitManager::open(std::path::Path::new(".")) {
                Ok(gm) => {
                    let branch = gm.current_branch().unwrap_or_else(|_| "unknown".into());
                    println!("  Repository : detected");
                    println!("  Branch     : {branch}");
                }
                Err(_) => {
                    println!("  Repository : not detected");
                }
            }
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
