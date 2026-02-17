//! Máquina de estados do WOLRAM — núcleo do ciclo de vida de jobs.
//!
//! Define o [`Job`], os [`State`]s (INIT, DEFINE_AGENT, PROCESS, END),
//! a [`StateMachine`] que calcula transições, e o [`AuditRecord`] produzido ao final.
//! Inclui também tipos auxiliares: [`ModelTier`], [`RetryConfig`], [`FailureKind`],
//! [`JobOutcome`] e [`JobStatus`].

mod job;
mod state;

pub use job::{
    AuditRecord, FailureKind, Job, JobOutcome, JobStatus, ModelTier, Priority, RetryConfig,
    TodoItem,
};
pub use state::{State, StateMachine, Transition};
