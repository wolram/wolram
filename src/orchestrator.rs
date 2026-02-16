use anyhow::{bail, Result};
use tokio::time::sleep;
use std::time::Duration;

use crate::router::{ModelSelector, SkillRouter};
use crate::state_machine::{
    AuditRecord, Job, JobOutcome, JobStatus, StateMachine, Transition,
};

/// Drives jobs through the full state machine lifecycle.
pub struct JobOrchestrator {
    /// Whether an Anthropic client is available for real API calls.
    pub has_client: bool,
    /// Whether git integration is available for auto-commits.
    pub has_git: bool,
}

impl Default for JobOrchestrator {
    fn default() -> Self {
        Self {
            has_client: false,
            has_git: false,
        }
    }
}

impl JobOrchestrator {
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
            let outcome = self.execute_process(job);
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
    fn execute_process(&self, _job: &Job) -> JobOutcome {
        if self.has_client {
            // TODO: call Anthropic API when client module is available
            JobOutcome::Success
        } else {
            // Simulate success
            JobOutcome::Success
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
        let orch = JobOrchestrator::default();
        let mut job = Job::new("Implement the user profile page".into(), RetryConfig::default());

        let record = orch.run_job(&mut job).await.unwrap();

        assert_eq!(record.status, JobStatus::Completed);
        assert_eq!(record.retry_count, 0);
        let agent = record.agent.unwrap();
        assert_eq!(agent.skill, "code_generation");
    }

    #[tokio::test]
    async fn orchestrator_rejects_empty_description() {
        let orch = JobOrchestrator::default();
        let mut job = Job::new("".into(), RetryConfig::default());

        let result = orch.run_job(&mut job).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn orchestrator_assigns_correct_skill_and_model() {
        let orch = JobOrchestrator::default();
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
        let orch = JobOrchestrator::default();
        let mut job = Job::new("Write tests for the parser".into(), RetryConfig::default());

        let record = orch.run_job(&mut job).await.unwrap();
        assert_eq!(
            record.state_transitions,
            vec![State::Init, State::DefineAgent, State::Process, State::End]
        );
    }
}
