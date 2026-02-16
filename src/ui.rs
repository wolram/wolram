//! Interface de terminal do WOLRAM — spinners e saída colorida.
//!
//! Usa as crates `indicatif` para spinners de progresso e `console` para
//! estilização com cores. O [`JobProgress`] acompanha visualmente
//! a execução de um job no terminal.

use console::Style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::state_machine::{AuditRecord, JobOutcome, JobStatus, State};

/// Indicador visual de progresso para a execução de um job no terminal.
///
/// Exibe um spinner animado durante o processamento e mensagens
/// coloridas para sucesso (verde), falha (vermelho) e retentativa (amarelo).
pub struct JobProgress {
    // Barra de progresso/spinner do indicatif.
    pb: ProgressBar,
    // Estilo verde para mensagens de sucesso.
    green: Style,
    // Estilo vermelho para mensagens de falha.
    red: Style,
    // Estilo amarelo para mensagens de retentativa.
    yellow: Style,
}

impl JobProgress {
    /// Inicia o spinner com a descrição do job e retorna a instância de progresso.
    pub fn start(description: &str) -> Self {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .expect("invalid template"),
        );
        pb.set_message(format!("INIT: {description}"));
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        Self {
            pb,
            green: Style::new().green().bold(),
            red: Style::new().red().bold(),
            yellow: Style::new().yellow(),
        }
    }

    /// Atualiza a mensagem do spinner para refletir o estado atual.
    #[allow(dead_code)]
    pub fn update_state(&self, state: State) {
        self.pb.set_message(format!("{state}"));
    }

    /// Exibe uma mensagem de retentativa com o número da tentativa e o motivo.
    #[allow(dead_code)]
    pub fn retry(&self, attempt: u32, max: u32, reason: &str) {
        self.pb.println(format!(
            "  {} Retry {attempt}/{max}: {reason}",
            self.yellow.apply_to("↻")
        ));
    }

    /// Finaliza o spinner e exibe o resultado final do job.
    ///
    /// Sucesso é mostrado em verde com checkmark; falha em vermelho com X.
    pub fn complete(&self, outcome: &JobOutcome) {
        self.pb.finish_and_clear();
        match outcome {
            JobOutcome::Success => {
                println!("  {} Job completed successfully", self.green.apply_to("✓"));
            }
            JobOutcome::Failure(kind) => {
                println!("  {} Job failed: {kind}", self.red.apply_to("✗"));
            }
        }
    }

    /// Imprime o registro de auditoria formatado em JSON com estilo colorido.
    pub fn print_audit(&self, record: &AuditRecord) {
        let status_style = match record.status {
            JobStatus::Completed => &self.green,
            JobStatus::Failed => &self.red,
            _ => &self.yellow,
        };
        println!();
        println!("{}", status_style.apply_to("─── Audit Record ───"));
        println!(
            "{}",
            serde_json::to_string_pretty(record).unwrap_or_default()
        );
    }
}
