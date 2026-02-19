//! Interface de terminal do WOLRAM — spinners e saída colorida.
//!
//! Usa as crates `indicatif` para spinners de progresso e `console` para
//! estilização com cores. O [`JobProgress`] acompanha visualmente
//! a execução de um job no terminal.

use console::Style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::state_machine::{AuditRecord, JobOutcome, JobStatus, State};

/// Conta o total de linhas em todos os arquivos `.rs` do diretório atual (recursivo).
/// Ignora `target/` e diretórios ocultos para evitar contar código gerado.
pub fn count_repo_lines() -> u64 {
    count_lines_in_dir(std::path::Path::new("."))
}

fn count_lines_in_dir(dir: &std::path::Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    let mut total = 0u64;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !name.starts_with('.') && name != "target" {
                total += count_lines_in_dir(&path);
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                total += content.lines().count() as u64;
            }
        }
    }
    total
}

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

    /// Imprime um rodapé compacto com custo estimado, duração e linhas de código do repositório.
    ///
    /// Corrige o bug de `duration_ms` negativo (clock skew) usando `.max(0)`.
    pub fn print_footer(&self, record: &AuditRecord, repo_lines: u64) {
        let cyan = Style::new().cyan();
        let bold = Style::new().bold();

        // Bug fix: duration_ms é i64 e pode ser negativo por ajuste de relógio.
        let duration_ms = record.duration_ms.max(0) as u64;
        let time_str = if duration_ms < 1_000 {
            format!("{duration_ms}ms")
        } else if duration_ms < 60_000 {
            format!("{:.1}s", duration_ms as f64 / 1_000.0)
        } else {
            format!("{}m {}s", duration_ms / 60_000, (duration_ms % 60_000) / 1_000)
        };

        println!();
        println!("{}", cyan.apply_to("─── Summary ─────────────────────────────────"));
        println!("  {}  ${:.4}", bold.apply_to("Cost :"), record.cost_usd);
        println!("  {}  {time_str}", bold.apply_to("Time :"));
        println!("  {}  {repo_lines} lines", bold.apply_to("Repo :"));
        println!("{}", cyan.apply_to("─────────────────────────────────────────────"));
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
