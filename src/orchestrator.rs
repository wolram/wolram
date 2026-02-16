use anyhow::{bail, Result};
use tokio::time::sleep;
use std::time::Duration;

use crate::anthropic::{AnthropicClient, AnthropicError, Message, MessagesRequest};
use crate::router::{ModelSelector, SkillRouter};
use crate::state_machine::{
    AuditRecord, FailureKind, Job, JobOutcome, JobStatus, ModelTier, StateMachine, Transition,
};

/// Drives jobs through the full state machine lifecycle.
pub struct JobOrchestrator {
    /// Optional Anthropic client for real API calls.
    pub client: Option<AnthropicClient>,
    /// Whether git integration is available for auto-commits.
    pub has_git: bool,
}

impl Default for JobOrchestrator {
    fn default() -> Self {
        Self {
            client: None,
            has_git: false,
        }
    }
}

/// Map a `ModelTier` to the Anthropic API model identifier string.
fn model_tier_to_api_string(tier: &ModelTier) -> &'static str {
    match tier {
        ModelTier::Haiku => "claude-haiku-4-5-20251001",
        ModelTier::Sonnet => "claude-sonnet-4-5-20250929",
        ModelTier::Opus => "claude-opus-4-6",
    }
}

impl JobOrchestrator {
    /// Create a new orchestrator with an optional Anthropic client.
    pub fn new(client: Option<AnthropicClient>, has_git: bool) -> Self {
        Self { client, has_git }
    }

    /// Run a job through all state machine phases, returning an audit record.
    pub async fn run_job(&self, job: &mut Job) -> Result<AuditRecord> {
        // INIT: validate and mark in-progress
        job.status = JobStatus::InProgress;
        if job.description.trim().is_empty() {
            bail!("Job description must not be empty");
        }
        let t = StateMachine::next(job, JobOutcome::Success);
        if !matches!(t, Transition::Next(_)) {
            bail!("Unexpected transition from Init: {t:?}");
        }

        // DEFINE_AGENT: route skill and select model
        let skill = SkillRouter::route(&job.description);
        let model = ModelSelector::select(&job.description);
        job.assign_agent(skill, model);
        let t = StateMachine::next(job, JobOutcome::Success);
        if !matches!(t, Transition::Next(_)) {
            bail!("Unexpected transition from DefineAgent: {t:?}");
        }

        // PROCESS: execute (with retry support)
        loop {
            let outcome = self.execute_process(job).await;
            let t = StateMachine::next(job, outcome);
            match t {
                Transition::Next(_) => break,     // → End
                Transition::Retry { reason, .. } => {
                    let delay_ms = job.retry_config.delay_for_attempt(job.retry_count);
                    log_retry(job.retry_count, job.retry_config.max_retries, &reason.to_string(), delay_ms);
                    sleep(Duration::from_millis(delay_ms)).await;
                }
                Transition::Complete(JobOutcome::Failure(kind)) => {
                    bail!("Job failed after {} retries: {kind}", job.retry_count);
                }
                Transition::Complete(JobOutcome::Success) => break,
            }
        }

        // END: produce audit record
        let record = AuditRecord::from_job(job);
        Ok(record)
    }

    /// Execute the process phase. Uses the API client if available, otherwise simulates success.
    async fn execute_process(&self, job: &Job) -> JobOutcome {
        let client = match &self.client {
            Some(c) => c,
            None => return JobOutcome::Success, // stub mode
        };

        let agent = match &job.agent {
            Some(a) => a,
            None => return JobOutcome::Failure(FailureKind::System("No agent assigned".into())),
        };

        let model = model_tier_to_api_string(&agent.model).to_string();
        let req = MessagesRequest {
            model,
            max_tokens: 4096,
            messages: vec![Message {
                role: "user".into(),
                content: format!(
                    "You are an AI coding assistant. Please perform the following task:\n\n{}",
                    job.description
                ),
            }],
        };

        match client.send_message(&req).await {
            Ok(_) => JobOutcome::Success,
            Err(AnthropicError::RateLimited { .. }) => {
                JobOutcome::Failure(FailureKind::System("Rate limited".into()))
            }
            Err(e) => JobOutcome::Failure(FailureKind::System(e.to_string())),
        }
    }
}

fn log_retry(attempt: u32, max: u32, reason: &str, delay_ms: u64) {
    eprintln!(
        "  ↻ Retry {attempt}/{max}: {reason} (waiting {delay_ms}ms)"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_machine::{Job, RetryConfig, State};

    #[tokio::test]
    async fn orchestrator_happy_path() {
        let orch = JobOrchestrator::new(None, false);
        let mut job = Job::new("Implement the user profile page".into(), RetryConfig::default());

        let record = orch.run_job(&mut job).await.unwrap();

        assert_eq!(record.status, JobStatus::Completed);
        assert_eq!(record.retry_count, 0);
        let agent = record.agent.unwrap();
        assert_eq!(agent.skill, "code_generation");
    }

    #[tokio::test]
    async fn orchestrator_rejects_empty_description() {
        let orch = JobOrchestrator::new(None, false);
        let mut job = Job::new("".into(), RetryConfig::default());

        let result = orch.run_job(&mut job).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn orchestrator_assigns_correct_skill_and_model() {
        let orch = JobOrchestrator::new(None, false);
        let mut job = Job::new(
            "Refactor the entire authentication module".into(),
            RetryConfig::default(),
        );

        let record = orch.run_job(&mut job).await.unwrap();
        let agent = record.agent.unwrap();
        assert_eq!(agent.skill, "refactoring");
        assert_eq!(agent.model, crate::state_machine::ModelTier::Opus);
    }

    #[tokio::test]
    async fn orchestrator_records_state_transitions() {
        let orch = JobOrchestrator::new(None, false);
        let mut job = Job::new("Write tests for the parser".into(), RetryConfig::default());

        let record = orch.run_job(&mut job).await.unwrap();
        assert_eq!(
            record.state_transitions,
            vec![State::Init, State::DefineAgent, State::Process, State::End]
        );
    }

    #[test]
    fn model_tier_mapping() {
        assert_eq!(model_tier_to_api_string(&ModelTier::Haiku), "claude-haiku-4-5-20251001");
        assert_eq!(model_tier_to_api_string(&ModelTier::Sonnet), "claude-sonnet-4-5-20250929");
        assert_eq!(model_tier_to_api_string(&ModelTier::Opus), "claude-opus-4-6");
    }
}
