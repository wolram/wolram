use std::fmt;

use serde::{Deserialize, Serialize};

use super::job::{FailureKind, Job, JobOutcome, JobStatus};

/// The four states of the WOLRAM job state machine.
///
/// Each job flows through: INIT → DEFINE_AGENT → PROCESS → END
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

/// The result of evaluating a state transition.
#[derive(Debug, Clone, PartialEq)]
pub enum Transition {
    /// Advance to the next state.
    Next(State),
    /// Retry the current state due to a failure.
    Retry { state: State, reason: FailureKind },
    /// The job has completed (successfully or with a terminal failure).
    Complete(JobOutcome),
}

/// Drives a `Job` through the state machine.
pub struct StateMachine;

impl StateMachine {
    /// Compute the next transition for the given job based on its current state
    /// and the provided outcome.
    ///
    /// - In `Init` and `DefineAgent`, success advances to the next state;
    ///   failure retries if retries remain, otherwise completes with failure.
    /// - In `Process`, same logic applies but a successful completion
    ///   transitions to `End`.
    /// - `End` is terminal and always returns `Complete`.
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

        // Apply the transition to the job.
        match &transition {
            Transition::Next(next_state) => {
                job.state_history.push(job.state);
                job.state = *next_state;
                if *next_state == State::End {
                    job.status = JobStatus::Completed;
                }
            }
            Transition::Retry { state, .. } => {
                // State stays the same; retry count was already incremented
                // in handle_failure.
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

        // End is terminal.
        let t = StateMachine::next(&mut job, JobOutcome::Success);
        assert_eq!(t, Transition::Complete(JobOutcome::Success));
    }

    #[test]
    fn business_failure_retries_then_fails() {
        let mut job = make_job(2);
        // Move to Process state.
        StateMachine::next(&mut job, JobOutcome::Success);
        StateMachine::next(&mut job, JobOutcome::Success);
        assert_eq!(job.state, State::Process);

        // First failure — retry 1/2.
        let t = StateMachine::next(
            &mut job,
            JobOutcome::Failure(FailureKind::Business("tests failed".into())),
        );
        assert!(matches!(t, Transition::Retry { .. }));
        assert_eq!(job.retry_count, 1);
        assert_eq!(job.state, State::Process);

        // Second failure — retry 2/2.
        let t = StateMachine::next(
            &mut job,
            JobOutcome::Failure(FailureKind::Business("tests failed again".into())),
        );
        assert!(matches!(t, Transition::Retry { .. }));
        assert_eq!(job.retry_count, 2);

        // Third failure — max retries exceeded, terminal failure.
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

        // Fail in Init with a system error.
        let t = StateMachine::next(
            &mut job,
            JobOutcome::Failure(FailureKind::System("API timeout".into())),
        );
        assert!(matches!(t, Transition::Retry { .. }));
        assert_eq!(job.retry_count, 1);

        // Second failure exceeds max retries.
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

        // Fail once in Init.
        let t = StateMachine::next(
            &mut job,
            JobOutcome::Failure(FailureKind::System("network error".into())),
        );
        assert!(matches!(t, Transition::Retry { .. }));
        assert_eq!(job.state, State::Init);

        // Succeed on retry.
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
