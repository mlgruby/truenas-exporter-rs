use anyhow::{Context, Result};
use secrecy::SecretString;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub truenas: TrueNasConfig,
    pub server: ServerConfig,
    pub metrics: MetricsConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TrueNasConfig {
    pub host: String,
    pub api_key: SecretString,
    #[serde(default = "default_use_tls")]
    pub use_tls: bool,
    #[serde(default = "default_verify_ssl")]
    #[allow(dead_code)]
    pub verify_ssl: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    #[serde(default = "default_addr")]
    pub addr: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MetricsConfig {
    #[serde(default = "default_scrape_interval")]
    pub scrape_interval_seconds: u64,
    #[serde(default = "default_true")]
    pub collect_pool_metrics: bool,
    #[serde(default = "default_true")]
    pub collect_system_metrics: bool,
}

fn default_addr() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    9100
}

fn default_use_tls() -> bool {
    false
}

fn default_verify_ssl() -> bool {
    true
}

fn default_scrape_interval() -> u64 {
    60
}

fn default_true() -> bool {
    true
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        // Load environment variables from .env if present
        dotenvy::dotenv().ok();

        let config = config::Config::builder()
            .add_source(config::File::with_name(path).required(false))
            .add_source(config::Environment::with_prefix("TRUENAS_EXPORTER").separator("__"))
            .build()
            .context("Failed to build configuration")?;

        config
            .try_deserialize()
            .context("Failed to deserialize configuration")
    }
}
