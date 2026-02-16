//! Configuração do WOLRAM carregada a partir de `wolram.toml`.
//!
//! A struct [`WolramConfig`] contém todos os parâmetros configuráveis.
//! Valores não presentes no arquivo usam defaults sensíveis.
//! A variável de ambiente `ANTHROPIC_API_KEY` tem precedência sobre o arquivo.

use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

/// Configuração de nível superior carregada de `wolram.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct WolramConfig {
    /// Chave da API Anthropic.
    #[serde(default)]
    pub api_key: String,

    /// Nível de modelo padrão quando não especificado via CLI.
    #[serde(default = "default_model_tier")]
    pub default_model_tier: String,

    /// Máximo de retentativas antes de marcar um job como falho.
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Atraso base em milissegundos para backoff exponencial.
    #[serde(default = "default_base_delay_ms")]
    pub base_delay_ms: u64,
}

// Valor padrão para o nível de modelo: "sonnet".
fn default_model_tier() -> String {
    "sonnet".to_string()
}

// Valor padrão para retentativas máximas: 3.
fn default_max_retries() -> u32 {
    3
}

// Valor padrão para o atraso base: 1000ms.
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
    /// Carrega a configuração de `wolram.toml` no diretório atual.
    /// Usa valores padrão se o arquivo não existir.
    pub fn load() -> Result<Self> {
        let path = Path::new("wolram.toml");
        let mut config = if path.exists() {
            let contents = std::fs::read_to_string(path)?;
            toml::from_str::<WolramConfig>(&contents)?
        } else {
            Self::default()
        };

        // Variável de ambiente tem precedência sobre o arquivo de configuração para a chave API.
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY")
            && !key.is_empty()
        {
            config.api_key = key;
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
        // No ambiente de teste, tipicamente não há wolram.toml no diretório de trabalho.
        let config = WolramConfig::load().unwrap();
        assert_eq!(config.max_retries, 3);
    }
}
