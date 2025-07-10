use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::sandbox::{SandboxManager, SandboxRequest, SandboxResponse};

pub mod handlers;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxFile {
    pub path: String,
    pub content: String,
    pub is_executable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSandboxRequest {
    pub runtime: String,
    pub code: String,
    pub entry_point: Option<String>,
    pub timeout_ms: Option<u64>,
    pub memory_limit_mb: Option<u64>,
    pub env_vars: Option<HashMap<String, String>>,
    pub files: Option<Vec<SandboxFile>>,
    pub mode: Option<String>, // "oneshot" or "persistent"
    pub install_deps: Option<bool>,
    pub dev_server: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxInfo {
    pub id: String,
    pub status: String,
    pub runtime: String,
    pub created_at: String,
    pub timeout_ms: u64,
    pub memory_limit_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub sandbox_id: String,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub execution_time_ms: u64,
}

pub type AppState = Arc<RwLock<SandboxManager>>;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/execute", post(handlers::execute_one_shot))
        .route("/sandbox", post(handlers::create_sandbox))
        .route("/sandbox/:id", get(handlers::get_sandbox))
        .route("/sandbox/:id", axum::routing::delete(handlers::delete_sandbox))
        .route("/sandbox/:id/execute", post(handlers::execute_code))
        .route("/sandbox", get(handlers::list_sandboxes))
        .route("/sandbox/:id/files", post(handlers::upload_files))
        .with_state(state)
}