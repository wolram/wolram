use anyhow::{bail, Result};
use tokio::time::sleep;
use std::time::Duration;

use crate::anthropic::{AnthropicClient, AnthropicError, Message, MessageSender, MessagesRequest};
use crate::router::{classify_with_llm, ModelSelector, SkillRouter};
use crate::state_machine::{
    AuditRecord, FailureKind, Job, JobOutcome, JobStatus, ModelTier, StateMachine, Transition,
};

/// Drives jobs through the full state machine lifecycle.
pub struct JobOrchestrator {
    /// Optional Anthropic client for real API calls.
    pub client: Option<AnthropicClient>,
    /// Whether git integration is available for auto-commits.
    pub has_git: bool,
    /// Optional CLI model override — bypasses model selection when set.
    pub model_override: Option<ModelTier>,
}

impl Default for JobOrchestrator {
    fn default() -> Self {
        Self {
            client: None,
            has_git: false,
            model_override: None,
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

/// Resolve skill and model using an LLM client with optional model override.
/// Tries LLM classification first, falls back to keyword scoring on failure.
pub async fn resolve_skill_and_model_with_client(
    client: &impl MessageSender,
    description: &str,
    model_override: Option<ModelTier>,
) -> (String, ModelTier) {
    if let Ok((skill, llm_model)) = classify_with_llm(client, description).await {
        let model = model_override.unwrap_or(llm_model);
        return (skill, model);
    }

    // Fallback to keyword scoring
    let skill = SkillRouter::route(description);
    let model = model_override.unwrap_or_else(|| ModelSelector::select(description));
    (skill, model)
}

/// Resolve skill and model using keyword scoring only (no LLM).
fn resolve_skill_and_model_keywords(
    description: &str,
    model_override: Option<ModelTier>,
) -> (String, ModelTier) {
    let skill = SkillRouter::route(description);
    let model = model_override.unwrap_or_else(|| ModelSelector::select(description));
    (skill, model)
}

impl JobOrchestrator {
    /// Create a new orchestrator with an optional Anthropic client.
    pub fn new(client: Option<AnthropicClient>, has_git: bool) -> Self {
        Self {
            client,
            has_git,
            model_override: None,
        }
    }

    /// Create a new orchestrator with a model override.
    pub fn with_model_override(
        client: Option<AnthropicClient>,
        has_git: bool,
        model_override: Option<ModelTier>,
    ) -> Self {
        Self {
            client,
            has_git,
            model_override,
        }
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
        let (skill, model) = if let Some(client) = &self.client {
            resolve_skill_and_model_with_client(client, &job.description, self.model_override.clone()).await
        } else {
            resolve_skill_and_model_keywords(&job.description, self.model_override.clone())
        };
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

    #[tokio::test]
    async fn orchestrator_model_override() {
        let orch = JobOrchestrator::with_model_override(None, false, Some(ModelTier::Opus));
        let mut job = Job::new("fix typo".into(), RetryConfig::default());

        let record = orch.run_job(&mut job).await.unwrap();
        let agent = record.agent.unwrap();
        assert_eq!(agent.model, ModelTier::Opus);
        assert_eq!(agent.skill, "bug_fix");
    }

    // --- Mock-based tests for resolve_skill_and_model_with_client ---

    use crate::anthropic::types::{ContentBlock, Usage};
    use crate::anthropic::{MessageSender, MessagesResponse};

    struct MockClient {
        response: Result<String, AnthropicError>,
    }

    impl MockClient {
        fn ok(text: &str) -> Self {
            Self { response: Ok(text.to_string()) }
        }
        fn err(e: AnthropicError) -> Self {
            Self { response: Err(e) }
        }
    }

    impl MessageSender for MockClient {
        async fn send_message(
            &self,
            _req: &crate::anthropic::MessagesRequest,
        ) -> Result<MessagesResponse, AnthropicError> {
            match &self.response {
                Ok(text) => Ok(MessagesResponse {
                    id: "mock".into(),
                    content: vec![ContentBlock {
                        content_type: "text".into(),
                        text: text.clone(),
                    }],
                    model: "mock".into(),
                    stop_reason: Some("end_turn".into()),
                    usage: Usage { input_tokens: 0, output_tokens: 0 },
                }),
                Err(_) => Err(AnthropicError::ApiError {
                    status: 500,
                    message: "mock error".to_string(),
                }),
            }
        }
    }

    #[tokio::test]
    async fn resolve_with_llm_success() {
        let client = MockClient::ok(r#"{"skill":"testing","complexity":"complex"}"#);
        let (skill, model) = resolve_skill_and_model_with_client(&client, "anything", None).await;
        assert_eq!(skill, "testing");
        assert_eq!(model, ModelTier::Opus);
    }

    #[tokio::test]
    async fn resolve_with_llm_fallback_on_error() {
        let client = MockClient::err(AnthropicError::ApiError {
            status: 500,
            message: "fail".into(),
        });
        let (skill, model) = resolve_skill_and_model_with_client(&client, "fix typo", None).await;
        assert_eq!(skill, "bug_fix");
        assert_eq!(model, ModelTier::Haiku);
    }

    #[tokio::test]
    async fn resolve_with_llm_and_model_override() {
        let client = MockClient::ok(r#"{"skill":"documentation","complexity":"simple"}"#);
        let (skill, model) = resolve_skill_and_model_with_client(&client, "write docs", Some(ModelTier::Opus)).await;
        assert_eq!(skill, "documentation");
        assert_eq!(model, ModelTier::Opus);
    }

    #[tokio::test]
    async fn resolve_with_llm_invalid_json_fallback() {
        let client = MockClient::ok("not valid json at all");
        let (skill, model) = resolve_skill_and_model_with_client(&client, "fix typo", None).await;
        assert_eq!(skill, "bug_fix");
        assert_eq!(model, ModelTier::Haiku);
    }

    #[tokio::test]
    async fn orchestrator_model_override_preserves_skill() {
        let orch = JobOrchestrator::with_model_override(None, false, Some(ModelTier::Haiku));
        let mut job = Job::new(
            "Refactor the entire authentication module".into(),
            RetryConfig::default(),
        );

        let record = orch.run_job(&mut job).await.unwrap();
        let agent = record.agent.unwrap();
        assert_eq!(agent.skill, "refactoring");
        assert_eq!(agent.model, ModelTier::Haiku);
    }
}
