use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::sandbox::backend::SandboxBackendType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub sandbox: SandboxConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origin: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub backend: SandboxBackendType,
    pub default_timeout_ms: u64,
    pub default_memory_limit_mb: u64,
    pub max_concurrent_sandboxes: usize,
    pub cleanup_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8070,
                cors_origin: None,
            },
            sandbox: SandboxConfig {
                backend: SandboxBackendType::Docker,
                default_timeout_ms: 30000,
                default_memory_limit_mb: 256,
                max_concurrent_sandboxes: 10,
                cleanup_interval_seconds: 300,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
            },
        }
    }
}

impl Config {
    pub fn from_file(path: &PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn from_env() -> Self {
        let mut config = Config::default();

        if let Ok(host) = std::env::var("SANDBOX_HOST") {
            config.server.host = host;
        }

        if let Ok(port) = std::env::var("SANDBOX_PORT") {
            if let Ok(port) = port.parse::<u16>() {
                config.server.port = port;
            }
        }

        if let Ok(backend) = std::env::var("SANDBOX_BACKEND") {
            config.sandbox.backend = match backend.to_lowercase().as_str() {
                "docker" => SandboxBackendType::Docker,
                "nsjail" => SandboxBackendType::Nsjail,
                _ => SandboxBackendType::Docker,
            };
        }

        if let Ok(timeout) = std::env::var("SANDBOX_TIMEOUT_MS") {
            if let Ok(timeout) = timeout.parse::<u64>() {
                config.sandbox.default_timeout_ms = timeout;
            }
        }

        if let Ok(memory) = std::env::var("SANDBOX_MEMORY_LIMIT_MB") {
            if let Ok(memory) = memory.parse::<u64>() {
                config.sandbox.default_memory_limit_mb = memory;
            }
        }

        if let Ok(level) = std::env::var("LOG_LEVEL") {
            config.logging.level = level;
        }

        config
    }
}