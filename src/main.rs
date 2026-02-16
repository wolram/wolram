mod anthropic;
mod git;
mod state_machine;

use state_machine::{
    AuditRecord, Job, JobOutcome, ModelTier, RetryConfig, StateMachine, Transition,
};

fn main() {
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
