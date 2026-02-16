use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::state::State;

/// O nível do modelo usado para um job, determinando custo e capacidade.
///
/// - Tarefas simples (renomear, formatar, edições pequenas) → Haiku
/// - Tarefas médias (implementar função, escrever testes) → Sonnet
/// - Tarefas complexas (arquitetura, refatoração multi-arquivo) → Opus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelTier {
    Haiku,
    Sonnet,
    Opus,
}

impl ModelTier {
    /// Custo aproximado por job em USD para este nível de modelo.
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

/// Configuração do agente atribuída durante a fase DEFINE_AGENT.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentConfig {
    /// O tipo de habilidade/agente (ex.: "code_generation", "refactoring", "testing").
    pub skill: String,
    /// O nível de modelo selecionado para este job.
    pub model: ModelTier,
}

/// Distingue entre falhas de lógica e falhas de infraestrutura.
///
/// Ambas são retriáveis, mas podem requerer estratégias de tratamento diferentes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureKind {
    /// Lógica da tarefa falhou (saída incorreta, erro de validação, testes falharam).
    Business(String),
    /// Infraestrutura falhou (timeout da API, rate limit, erro de rede).
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

/// O resultado da execução de um estágio do job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobOutcome {
    Success,
    Failure(FailureKind),
}

/// Acompanha o status do ciclo de vida de um job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// Configuração do comportamento de retentativa.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Número máximo de retentativas antes de marcar o job como falho.
    pub max_retries: u32,
    /// Atraso base em milissegundos para backoff exponencial.
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
    /// Calcula o atraso para uma determinada tentativa usando backoff exponencial.
    ///
    /// Fórmula: `atraso = base_delay_ms * 2^(tentativa - 1)`
    pub fn delay_for_attempt(&self, attempt: u32) -> u64 {
        self.base_delay_ms * 2u64.pow(attempt.saturating_sub(1))
    }
}

/// Uma única tarefa na fila de jobs do WOLRAM.
///
/// O job percorre os estados INIT → DEFINE_AGENT → PROCESS → END,
/// acumulando histórico de transições e informações do agente atribuído.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub description: String,
    pub status: JobStatus,
    pub state: State,
    pub state_history: Vec<State>,
    pub retry_count: u32,
    pub retry_config: RetryConfig,
    /// Configuração do agente atribuída durante a fase DEFINE_AGENT.
    pub agent: Option<AgentConfig>,
    /// Texto da resposta do LLM da fase PROCESS (None em modo stub).
    #[serde(default)]
    pub llm_response: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Job {
    /// Cria um novo job com a descrição e configuração de retentativa fornecidas.
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
            llm_response: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Atribui uma configuração de agente a este job (tipicamente durante DEFINE_AGENT).
    pub fn assign_agent(&mut self, skill: String, model: ModelTier) {
        self.agent = Some(AgentConfig { skill, model });
    }

    /// Custo estimado em USD baseado no nível do modelo atribuído.
    /// Retorna 0.0 se nenhum agente foi atribuído ainda.
    pub fn estimated_cost_usd(&self) -> f64 {
        self.agent
            .as_ref()
            .map(|a| a.model.estimated_cost_usd())
            .unwrap_or(0.0)
    }
}

/// Registro de auditoria estruturado produzido ao final de um job.
///
/// Contém todas as informações relevantes para rastreabilidade: identificadores,
/// transições de estado, configuração do agente, contagem de retentativas,
/// custo estimado e timestamps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    pub job_id: String,
    pub description: String,
    pub status: JobStatus,
    pub state_transitions: Vec<State>,
    /// Habilidade e modelo do agente utilizados, se atribuídos.
    pub agent: Option<AgentConfig>,
    pub retry_count: u32,
    pub max_retries: u32,
    /// Custo estimado em USD baseado no nível do modelo.
    pub cost_usd: f64,
    /// Texto da resposta do LLM, se disponível.
    #[serde(default)]
    pub llm_response: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub duration_ms: i64,
}

impl AuditRecord {
    /// Gera um registro de auditoria a partir de um job concluído ou falho.
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
            llm_response: job.llm_response.clone(),
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
