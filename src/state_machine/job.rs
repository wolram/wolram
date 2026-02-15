use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::state::State;

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
            created_at: now,
            updated_at: now,
        }
    }
}

/// Structured audit record produced at job completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    pub job_id: String,
    pub description: String,
    pub status: JobStatus,
    pub state_transitions: Vec<State>,
    pub retry_count: u32,
    pub max_retries: u32,
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
            retry_count: job.retry_count,
            max_retries: job.retry_config.max_retries,
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
    }

    #[test]
    fn failure_kind_display() {
        let biz = FailureKind::Business("tests failed".into());
        assert_eq!(biz.to_string(), "Business failure: tests failed");

        let sys = FailureKind::System("API timeout".into());
        assert_eq!(sys.to_string(), "System failure: API timeout");
    }

    #[test]
    fn job_serialization_roundtrip() {
        let job = Job::new("Serialize me".into(), RetryConfig::default());
        let json = serde_json::to_string(&job).unwrap();
        let deserialized: Job = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, job.id);
        assert_eq!(deserialized.description, "Serialize me");
        assert_eq!(deserialized.state, State::Init);
    }
}
