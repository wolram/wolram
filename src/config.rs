use crate::error::WolramError;
use serde::Deserialize;
use std::path::{Path, PathBuf};

const CONFIG_FILE: &str = "wolram.toml";
const DEFAULT_MAX_RETRIES: u32 = 3;
const DEFAULT_MODEL: &str = "claude-sonnet-4-5-20250929";

#[derive(Debug, Clone)]
pub struct Config {
    pub api_key: String,
    pub default_model: String,
    pub max_retries: u32,
    pub project_dir: PathBuf,
    pub wolram_dir: PathBuf,
}

#[derive(Debug, Deserialize, Default)]
struct FileConfig {
    api_key: Option<String>,
    default_model: Option<String>,
    max_retries: Option<u32>,
}

impl Config {
    pub fn load() -> Result<Self, WolramError> {
        let project_dir = std::env::current_dir()?;
        let config_path = project_dir.join(CONFIG_FILE);

        let file_config = if config_path.exists() {
            let contents = std::fs::read_to_string(&config_path)?;
            toml::from_str::<FileConfig>(&contents)?
        } else {
            FileConfig::default()
        };

        let api_key = file_config
            .api_key
            .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
            .ok_or_else(|| {
                WolramError::Config(
                    "No API key found. Set ANTHROPIC_API_KEY env var or api_key in wolram.toml"
                        .into(),
                )
            })?;

        let default_model = file_config
            .default_model
            .or_else(|| std::env::var("WOLRAM_MODEL").ok())
            .unwrap_or_else(|| DEFAULT_MODEL.to_string());

        let max_retries = file_config
            .max_retries
            .unwrap_or(DEFAULT_MAX_RETRIES);

        let wolram_dir = project_dir.join(".wolram");

        Ok(Config {
            api_key,
            default_model,
            max_retries,
            project_dir,
            wolram_dir,
        })
    }

    pub fn jobs_path(&self) -> PathBuf {
        self.wolram_dir.join("jobs.json")
    }

    pub fn audit_dir(&self) -> PathBuf {
        self.wolram_dir.join("audit")
    }

    pub fn ensure_dirs(&self) -> Result<(), WolramError> {
        std::fs::create_dir_all(&self.wolram_dir)?;
        std::fs::create_dir_all(self.audit_dir())?;
        Ok(())
    }
}
