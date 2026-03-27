use anyhow::{bail, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub finnhub: FinnhubConfig,
    pub gemini: GeminiConfig,
    #[serde(default)]
    pub coinmarketcap: CoinMarketCapConfig,
    #[serde(default)]
    pub exchange_rate: ExchangeRateConfig,
    #[serde(default)]
    pub pdf: PdfConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub database_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FinnhubConfig {
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub api_key_env: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GeminiConfig {
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub api_key_env: Option<String>,
    pub model: String,
}

fn resolve_key(direct: &Option<String>, env_name: &Option<String>, label: &str) -> Result<String> {
    if let Some(key) = direct {
        if !key.is_empty() {
            return Ok(key.clone());
        }
    }
    if let Some(env_var) = env_name {
        if !env_var.is_empty() {
            return std::env::var(env_var)
                .map_err(|_| anyhow::anyhow!("{}: env var '{}' is not set", label, env_var));
        }
    }
    bail!("{}: neither api_key nor api_key_env is configured", label)
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PdfConfig {
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub password_env: Option<String>,
}

impl PdfConfig {
    pub fn resolve_password(&self) -> Option<String> {
        if let Some(pw) = &self.password {
            if !pw.is_empty() {
                return Some(pw.clone());
            }
        }
        if let Some(env_var) = &self.password_env {
            if !env_var.is_empty() {
                return std::env::var(env_var).ok();
            }
        }
        None
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CoinMarketCapConfig {
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub api_key_env: Option<String>,
}

impl CoinMarketCapConfig {
    pub fn resolve_api_key(&self) -> Result<String> {
        resolve_key(&self.api_key, &self.api_key_env, "coinmarketcap")
    }
}

fn default_base_currency() -> String {
    "USD".to_string()
}

fn default_target_currencies() -> Vec<String> {
    vec!["TWD".to_string()]
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExchangeRateConfig {
    #[serde(default = "default_base_currency")]
    pub base: String,
    #[serde(default = "default_target_currencies")]
    pub currencies: Vec<String>,
}

impl Default for ExchangeRateConfig {
    fn default() -> Self {
        Self {
            base: default_base_currency(),
            currencies: default_target_currencies(),
        }
    }
}

impl FinnhubConfig {
    pub fn resolve_api_key(&self) -> Result<String> {
        resolve_key(&self.api_key, &self.api_key_env, "finnhub")
    }
}

impl GeminiConfig {
    pub fn resolve_api_key(&self) -> Result<String> {
        resolve_key(&self.api_key, &self.api_key_env, "gemini")
    }
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn load_default() -> Result<Self> {
        // Priority: KABU_CONFIG env > XDG config > ./config.toml
        if let Ok(path) = std::env::var("KABU_CONFIG") {
            return Self::load(&path);
        }

        if let Some(config_dir) = dirs::config_dir() {
            let path = config_dir.join("kabu/config.toml");
            if path.exists() {
                return Self::load(&path);
            }
        }

        Self::load("config.toml")
    }
}
