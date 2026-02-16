use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::state::State;

/// The model tier used for a job, determining cost and capability.
///
/// - Simple tasks (rename, format, small edits) → Haiku
/// - Medium tasks (implement function, write tests) → Sonnet
/// - Complex tasks (architecture, multi-file refactor) → Opus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelTier {
    Haiku,
    Sonnet,
    Opus,
}

impl ModelTier {
    /// Approximate cost per job in USD for this model tier.
    pub fn estimated_cost_usd(&self) -> f64 {
        match self {
            ModelTier::Haiku => 0.001,
            ModelTier::Sonnet => 0.005,
            ModelTier::Opus => 0.05,
        }
    }
}

impl std::fmt::Display for ModelTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelTier::Haiku => write!(f, "haiku"),
            ModelTier::Sonnet => write!(f, "sonnet"),
            ModelTier::Opus => write!(f, "opus"),
        }
    }
}

/// Agent configuration assigned during the DEFINE_AGENT phase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentConfig {
    /// The skill/agent type (e.g. "code_generation", "refactoring", "testing").
    pub skill: String,
    /// The model tier selected for this job.
    pub model: ModelTier,
}

/// Distinguishes between logic failures and infrastructure failures.
/// Both are retryable, but may warrant different handling strategies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureKind {
    /// Task logic failed (wrong output, validation error, tests fail).
    Business(String),
    /// Infrastructure failed (API timeout, rate limit, network error).
    System(String),
}

impl std::fmt::Display for FailureKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FailureKind::Business(msg) => write!(f, "Business failure: {msg}"),
            FailureKind::System(msg) => write!(f, "System failure: {msg}"),
        }
    }
}

/// The result of executing a job stage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobOutcome {
    Success,
    Failure(FailureKind),
}

/// Tracks the lifecycle status of a job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// Configuration for retry behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retries before marking a job as failed.
    pub max_retries: u32,
    /// Base delay in milliseconds for exponential backoff.
    pub base_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000,
        }
    }
}

impl RetryConfig {
    /// Calculate the delay for a given retry attempt using exponential backoff.
    /// delay = base_delay_ms * 2^(attempt - 1)
    pub fn delay_for_attempt(&self, attempt: u32) -> u64 {
        self.base_delay_ms * 2u64.pow(attempt.saturating_sub(1))
    }
}

/// A single task in the WOLRAM job queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub description: String,
    pub status: JobStatus,
    pub state: State,
    pub state_history: Vec<State>,
    pub retry_count: u32,
    pub retry_config: RetryConfig,
    /// Agent configuration assigned during the DEFINE_AGENT phase.
    pub agent: Option<AgentConfig>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Job {
    pub fn new(description: String, retry_config: RetryConfig) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            description,
            status: JobStatus::Pending,
            state: State::Init,
            state_history: Vec::new(),
            retry_count: 0,
            retry_config,
            agent: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Assign an agent configuration to this job (typically during DEFINE_AGENT).
    pub fn assign_agent(&mut self, skill: String, model: ModelTier) {
        self.agent = Some(AgentConfig { skill, model });
    }

    /// Estimated cost in USD based on the assigned model tier.
    /// Returns 0.0 if no agent has been assigned yet.
    pub fn estimated_cost_usd(&self) -> f64 {
        self.agent
            .as_ref()
            .map(|a| a.model.estimated_cost_usd())
            .unwrap_or(0.0)
    }
}

/// Structured audit record produced at job completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    pub job_id: String,
    pub description: String,
    pub status: JobStatus,
    pub state_transitions: Vec<State>,
    /// Agent skill and model used, if assigned.
    pub agent: Option<AgentConfig>,
    pub retry_count: u32,
    pub max_retries: u32,
    /// Estimated cost in USD based on the model tier.
    pub cost_usd: f64,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub duration_ms: i64,
}

impl AuditRecord {
    /// Generate an audit record from a completed or failed job.
    pub fn from_job(job: &Job) -> Self {
        let now = Utc::now();
        let duration = now - job.created_at;
        let mut transitions = job.state_history.clone();
        transitions.push(job.state);

        Self {
            job_id: job.id.clone(),
            description: job.description.clone(),
            status: job.status,
            state_transitions: transitions,
            agent: job.agent.clone(),
            retry_count: job.retry_count,
            max_retries: job.retry_config.max_retries,
            cost_usd: job.estimated_cost_usd(),
            started_at: job.created_at,
            completed_at: now,
            duration_ms: duration.num_milliseconds(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_creation_defaults() {
        let job = Job::new("Test".into(), RetryConfig::default());
        assert_eq!(job.status, JobStatus::Pending);
        assert_eq!(job.state, State::Init);
        assert_eq!(job.retry_count, 0);
        assert_eq!(job.retry_config.max_retries, 3);
        assert!(job.state_history.is_empty());
    }

    #[test]
    fn retry_config_exponential_backoff() {
        let config = RetryConfig {
            max_retries: 5,
            base_delay_ms: 1000,
        };
        assert_eq!(config.delay_for_attempt(1), 1000);
        assert_eq!(config.delay_for_attempt(2), 2000);
        assert_eq!(config.delay_for_attempt(3), 4000);
        assert_eq!(config.delay_for_attempt(4), 8000);
    }

    #[test]
    fn audit_record_from_job() {
        let job = Job::new("Implement feature".into(), RetryConfig::default());
        let record = AuditRecord::from_job(&job);

        assert_eq!(record.job_id, job.id);
        assert_eq!(record.description, "Implement feature");
        assert_eq!(record.retry_count, 0);
        assert_eq!(record.max_retries, 3);
        assert_eq!(record.state_transitions, vec![State::Init]);
        assert_eq!(record.agent, None);
        assert_eq!(record.cost_usd, 0.0);
    }

    #[test]
    fn audit_record_includes_agent_and_cost() {
        let mut job = Job::new("Implement feature".into(), RetryConfig::default());
        job.assign_agent("code_generation".to_string(), ModelTier::Sonnet);

        let record = AuditRecord::from_job(&job);

        let agent = record.agent.unwrap();
        assert_eq!(agent.skill, "code_generation");
        assert_eq!(agent.model, ModelTier::Sonnet);
        assert_eq!(record.cost_usd, 0.005);
    }

    #[test]
    fn failure_kind_display() {
        let biz = FailureKind::Business("tests failed".into());
        assert_eq!(biz.to_string(), "Business failure: tests failed");

        let sys = FailureKind::System("API timeout".into());
        assert_eq!(sys.to_string(), "System failure: API timeout");
    }

    #[test]
    fn model_tier_costs() {
        assert_eq!(ModelTier::Haiku.estimated_cost_usd(), 0.001);
        assert_eq!(ModelTier::Sonnet.estimated_cost_usd(), 0.005);
        assert_eq!(ModelTier::Opus.estimated_cost_usd(), 0.05);
    }

    #[test]
    fn model_tier_display() {
        assert_eq!(ModelTier::Haiku.to_string(), "haiku");
        assert_eq!(ModelTier::Sonnet.to_string(), "sonnet");
        assert_eq!(ModelTier::Opus.to_string(), "opus");
    }

    #[test]
    fn assign_agent_to_job() {
        let mut job = Job::new("Test".into(), RetryConfig::default());
        assert!(job.agent.is_none());
        assert_eq!(job.estimated_cost_usd(), 0.0);

        job.assign_agent("refactoring".to_string(), ModelTier::Opus);

        let agent = job.agent.as_ref().unwrap();
        assert_eq!(agent.skill, "refactoring");
        assert_eq!(agent.model, ModelTier::Opus);
        assert_eq!(job.estimated_cost_usd(), 0.05);
    }

    #[test]
    fn job_serialization_roundtrip() {
        let mut job = Job::new("Serialize me".into(), RetryConfig::default());
        job.assign_agent("testing".to_string(), ModelTier::Haiku);

        let json = serde_json::to_string(&job).unwrap();
        let deserialized: Job = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, job.id);
        assert_eq!(deserialized.description, "Serialize me");
        assert_eq!(deserialized.state, State::Init);
        let agent = deserialized.agent.unwrap();
        assert_eq!(agent.skill, "testing");
        assert_eq!(agent.model, ModelTier::Haiku);
    }
}
