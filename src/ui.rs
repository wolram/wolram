use console::Style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::state_machine::{AuditRecord, JobOutcome, JobStatus, State};

pub struct JobProgress {
    pb: ProgressBar,
    green: Style,
    red: Style,
    yellow: Style,
}

impl JobProgress {
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

    pub fn update_state(&self, state: State) {
        self.pb.set_message(format!("{state}"));
    }

    pub fn retry(&self, attempt: u32, max: u32, reason: &str) {
        self.pb.println(format!(
            "  {} Retry {attempt}/{max}: {reason}",
            self.yellow.apply_to("↻")
        ));
    }

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
