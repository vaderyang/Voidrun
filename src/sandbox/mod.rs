use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::api::SandboxInfo;

pub mod backend;
pub mod manager;

pub use backend::{SandboxBackend, SandboxBackendType};
pub use manager::SandboxManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxFile {
    pub path: String,
    pub content: String,
    pub is_executable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SandboxMode {
    OneShot,    // Execute once and cleanup (default)
    Persistent, // Keep running until explicitly stopped
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxRequest {
    pub id: String,
    pub runtime: String,
    pub code: String,
    pub entry_point: Option<String>,
    pub timeout_ms: u64,
    pub memory_limit_mb: u64,
    pub env_vars: HashMap<String, String>,
    pub files: Option<Vec<SandboxFile>>,
    pub mode: Option<SandboxMode>,
    pub install_deps: Option<bool>,
    pub dev_server: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxResponse {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub execution_time_ms: u64,
    pub is_running: Option<bool>,
    pub dev_server_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Sandbox {
    pub id: String,
    pub request: SandboxRequest,
    pub backend_type: SandboxBackendType,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub status: SandboxStatus,
    pub container_id: Option<String>,
    pub dev_server_port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SandboxStatus {
    Created,
    Installing,
    Running,
    DevServer,
    Completed,
    Failed,
    Terminated,
}

impl Sandbox {
    pub fn new(request: SandboxRequest, backend_type: SandboxBackendType) -> Self {
        Self {
            id: request.id.clone(),
            request,
            backend_type,
            created_at: chrono::Utc::now(),
            status: SandboxStatus::Created,
            container_id: None,
            dev_server_port: None,
        }
    }

    pub fn to_info(&self) -> SandboxInfo {
        SandboxInfo {
            id: self.id.clone(),
            status: format!("{:?}", self.status),
            runtime: self.request.runtime.clone(),
            created_at: self.created_at.to_rfc3339(),
            timeout_ms: self.request.timeout_ms,
            memory_limit_mb: self.request.memory_limit_mb,
        }
    }
}