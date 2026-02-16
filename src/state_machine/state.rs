use std::fmt;

use serde::{Deserialize, Serialize};

use super::job::{FailureKind, Job, JobOutcome, JobStatus};

/// Os quatro estados da máquina de estados de jobs do WOLRAM.
///
/// Cada job flui por: INIT → DEFINE_AGENT → PROCESS → END
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum State {
    Init,
    DefineAgent,
    Process,
    End,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            State::Init => write!(f, "INIT"),
            State::DefineAgent => write!(f, "DEFINE_AGENT"),
            State::Process => write!(f, "PROCESS"),
            State::End => write!(f, "END"),
        }
    }
}

/// O resultado da avaliação de uma transição de estado.
#[derive(Debug, Clone, PartialEq)]
pub enum Transition {
    /// Avança para o próximo estado.
    Next(State),
    /// Retenta o estado atual devido a uma falha.
    Retry { state: State, reason: FailureKind },
    /// O job foi concluído (com sucesso ou com falha terminal).
    Complete(JobOutcome),
}

/// Conduz um [`Job`] pela máquina de estados.
///
/// A struct não possui estado interno — todas as mutações acontecem
/// diretamente no [`Job`] passado por referência mutável.
pub struct StateMachine;

impl StateMachine {
    /// Calcula a próxima transição para o job dado, baseado no estado atual
    /// e no resultado fornecido.
    ///
    /// - Em `Init` e `DefineAgent`, sucesso avança para o próximo estado;
    ///   falha retenta se há retentativas restantes, caso contrário completa com falha.
    /// - Em `Process`, a mesma lógica se aplica, mas uma conclusão bem-sucedida
    ///   transiciona para `End`.
    /// - `End` é terminal e sempre retorna `Complete`.
    pub fn next(job: &mut Job, outcome: JobOutcome) -> Transition {
        let transition = match job.state {
            State::Init => match &outcome {
                JobOutcome::Success => Transition::Next(State::DefineAgent),
                JobOutcome::Failure(kind) => Self::handle_failure(job, kind.clone()),
            },
            State::DefineAgent => match &outcome {
                JobOutcome::Success => Transition::Next(State::Process),
                JobOutcome::Failure(kind) => Self::handle_failure(job, kind.clone()),
            },
            State::Process => match &outcome {
                JobOutcome::Success => Transition::Next(State::End),
                JobOutcome::Failure(kind) => Self::handle_failure(job, kind.clone()),
            },
            State::End => Transition::Complete(JobOutcome::Success),
        };

        // Aplica a transição ao job.
        match &transition {
            Transition::Next(next_state) => {
                job.state_history.push(job.state);
                job.state = *next_state;
                // Se o próximo estado é End, marca o job como concluído.
                if *next_state == State::End {
                    job.status = JobStatus::Completed;
                }
            }
            Transition::Retry { state, .. } => {
                // Estado permanece o mesmo; o contador de retentativas
                // já foi incrementado em handle_failure.
                job.state_history.push(*state);
            }
            Transition::Complete(outcome) => {
                job.state_history.push(job.state);
                job.status = match outcome {
                    JobOutcome::Success => JobStatus::Completed,
                    JobOutcome::Failure(_) => JobStatus::Failed,
                };
            }
        }

        transition
    }

    /// Trata uma falha: incrementa retentativas e decide entre retentar ou concluir.
    fn handle_failure(job: &mut Job, kind: FailureKind) -> Transition {
        job.retry_count += 1;
        if job.retry_count <= job.retry_config.max_retries {
            Transition::Retry {
                state: job.state,
                reason: kind,
            }
        } else {
            Transition::Complete(JobOutcome::Failure(kind))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_machine::job::RetryConfig;

    fn make_job(max_retries: u32) -> Job {
        Job::new(
            "Test job".to_string(),
            RetryConfig {
                max_retries,
                ..Default::default()
            },
        )
    }

    #[test]
    fn happy_path_walks_all_states() {
        let mut job = make_job(3);
        assert_eq!(job.state, State::Init);

        let t = StateMachine::next(&mut job, JobOutcome::Success);
        assert_eq!(t, Transition::Next(State::DefineAgent));
        assert_eq!(job.state, State::DefineAgent);

        let t = StateMachine::next(&mut job, JobOutcome::Success);
        assert_eq!(t, Transition::Next(State::Process));
        assert_eq!(job.state, State::Process);

        let t = StateMachine::next(&mut job, JobOutcome::Success);
        assert_eq!(t, Transition::Next(State::End));
        assert_eq!(job.state, State::End);
        assert_eq!(job.status, JobStatus::Completed);

        // End é terminal.
        let t = StateMachine::next(&mut job, JobOutcome::Success);
        assert_eq!(t, Transition::Complete(JobOutcome::Success));
    }

    #[test]
    fn business_failure_retries_then_fails() {
        let mut job = make_job(2);
        // Avança até o estado Process.
        StateMachine::next(&mut job, JobOutcome::Success);
        StateMachine::next(&mut job, JobOutcome::Success);
        assert_eq!(job.state, State::Process);

        // Primeira falha — retentativa 1/2.
        let t = StateMachine::next(
            &mut job,
            JobOutcome::Failure(FailureKind::Business("tests failed".into())),
        );
        assert!(matches!(t, Transition::Retry { .. }));
        assert_eq!(job.retry_count, 1);
        assert_eq!(job.state, State::Process);

        // Segunda falha — retentativa 2/2.
        let t = StateMachine::next(
            &mut job,
            JobOutcome::Failure(FailureKind::Business("tests failed again".into())),
        );
        assert!(matches!(t, Transition::Retry { .. }));
        assert_eq!(job.retry_count, 2);

        // Terceira falha — retentativas máximas excedidas, falha terminal.
        let t = StateMachine::next(
            &mut job,
            JobOutcome::Failure(FailureKind::Business("still failing".into())),
        );
        assert_eq!(
            t,
            Transition::Complete(JobOutcome::Failure(FailureKind::Business(
                "still failing".into()
            )))
        );
        assert_eq!(job.status, JobStatus::Failed);
    }

    #[test]
    fn system_failure_retries_then_fails() {
        let mut job = make_job(1);

        // Falha em Init com um erro de sistema.
        let t = StateMachine::next(
            &mut job,
            JobOutcome::Failure(FailureKind::System("API timeout".into())),
        );
        assert!(matches!(t, Transition::Retry { .. }));
        assert_eq!(job.retry_count, 1);

        // Segunda falha excede retentativas máximas.
        let t = StateMachine::next(
            &mut job,
            JobOutcome::Failure(FailureKind::System("API timeout".into())),
        );
        assert_eq!(
            t,
            Transition::Complete(JobOutcome::Failure(FailureKind::System(
                "API timeout".into()
            )))
        );
        assert_eq!(job.status, JobStatus::Failed);
    }

    #[test]
    fn zero_retries_fails_immediately() {
        let mut job = make_job(0);

        let t = StateMachine::next(
            &mut job,
            JobOutcome::Failure(FailureKind::Business("bad output".into())),
        );
        assert_eq!(
            t,
            Transition::Complete(JobOutcome::Failure(FailureKind::Business(
                "bad output".into()
            )))
        );
        assert_eq!(job.status, JobStatus::Failed);
    }

    #[test]
    fn retry_then_succeed() {
        let mut job = make_job(3);

        // Falha uma vez em Init.
        let t = StateMachine::next(
            &mut job,
            JobOutcome::Failure(FailureKind::System("network error".into())),
        );
        assert!(matches!(t, Transition::Retry { .. }));
        assert_eq!(job.state, State::Init);

        // Sucesso na retentativa.
        let t = StateMachine::next(&mut job, JobOutcome::Success);
        assert_eq!(t, Transition::Next(State::DefineAgent));
        assert_eq!(job.state, State::DefineAgent);
    }

    #[test]
    fn state_history_is_recorded() {
        let mut job = make_job(3);

        StateMachine::next(&mut job, JobOutcome::Success);
        StateMachine::next(&mut job, JobOutcome::Success);
        StateMachine::next(&mut job, JobOutcome::Success);

        assert_eq!(
            job.state_history,
            vec![State::Init, State::DefineAgent, State::Process]
        );
    }

    #[test]
    fn state_display() {
        assert_eq!(State::Init.to_string(), "INIT");
        assert_eq!(State::DefineAgent.to_string(), "DEFINE_AGENT");
        assert_eq!(State::Process.to_string(), "PROCESS");
        assert_eq!(State::End.to_string(), "END");
    }
}
