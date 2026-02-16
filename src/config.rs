use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

/// Top-level configuration loaded from `wolram.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct WolramConfig {
    /// Anthropic API key.
    #[serde(default)]
    pub api_key: String,

    /// Default model tier when not specified via CLI.
    #[serde(default = "default_model_tier")]
    pub default_model_tier: String,

    /// Maximum retries before marking a job as failed.
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Base delay in milliseconds for exponential backoff.
    #[serde(default = "default_base_delay_ms")]
    pub base_delay_ms: u64,
}

fn default_model_tier() -> String {
    "sonnet".to_string()
}

fn default_max_retries() -> u32 {
    3
}

fn default_base_delay_ms() -> u64 {
    1000
}

impl Default for WolramConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            default_model_tier: default_model_tier(),
            max_retries: default_max_retries(),
            base_delay_ms: default_base_delay_ms(),
        }
    }
}

impl WolramConfig {
    /// Load configuration from `wolram.toml` in the current directory.
    /// Falls back to defaults if the file doesn't exist.
    pub fn load() -> Result<Self> {
        let path = Path::new("wolram.toml");
        let mut config = if path.exists() {
            let contents = std::fs::read_to_string(path)?;
            toml::from_str::<WolramConfig>(&contents)?
        } else {
            Self::default()
        };

        // Env var overrides config file for API key.
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            if !key.is_empty() {
                config.api_key = key;
            }
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let config = WolramConfig::default();
        assert_eq!(config.default_model_tier, "sonnet");
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.base_delay_ms, 1000);
        assert!(config.api_key.is_empty());
    }

    #[test]
    fn deserialize_partial_toml() {
        let toml_str = r#"
            api_key = "sk-test-123"
            max_retries = 5
        "#;
        let config: WolramConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.api_key, "sk-test-123");
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.default_model_tier, "sonnet");
        assert_eq!(config.base_delay_ms, 1000);
    }

    #[test]
    fn load_falls_back_to_defaults() {
        // In test environment there's no wolram.toml at CWD typically
        let config = WolramConfig::load().unwrap();
        assert_eq!(config.max_retries, 3);
    }
}
