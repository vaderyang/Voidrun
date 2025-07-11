use axum::{
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use crate::sandbox::manager::SandboxManager;

pub mod handlers;
pub mod ui;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    pub uptime: u64,
    pub active_sandboxes: u32,
    pub total_sandboxes_created: u32,
    pub backend_type: String,
    pub version: String,
    pub memory_usage: ResourceUsage,
    pub cpu_usage: ResourceUsage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub used: f64,
    pub total: f64,
    pub percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SandboxInfo {
    pub id: String,
    pub status: String,
    pub runtime: String,
    pub created_at: String,
    pub uptime: u64,
    pub memory_mb: u64,
    pub cpu_percentage: f64,
    pub dev_server_url: Option<String>,
    pub allocated_port: Option<u16>,
    pub is_persistent: bool,
    pub container_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub sandbox_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LogQuery {
    pub lines: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct ApiEndpoint {
    pub method: String,
    pub path: String,
    pub description: String,
    pub parameters: Vec<ApiParameter>,
    pub example_request: Option<String>,
    pub example_response: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiParameter {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiTestRequest {
    pub method: String,
    pub path: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiTestResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub duration_ms: u64,
}

pub fn create_admin_router(app_state: Arc<RwLock<SandboxManager>>) -> Router {
    Router::new()
        .route("/admin", get(handlers::admin_ui))
        .route("/admin/api/status", get(handlers::get_system_status))
        .route("/admin/api/sandboxes", get(handlers::list_sandboxes))
        .route("/admin/api/sandboxes/:id", get(handlers::get_sandbox_info))
        .route("/admin/api/sandboxes/:id/logs", get(handlers::get_sandbox_logs))
        .route("/admin/api/sandboxes/:id/force-stop", post(handlers::force_stop_sandbox))
        .route("/admin/api/sandboxes/:id/resources", get(handlers::get_sandbox_resources))
        .route("/admin/api/logs", get(handlers::get_system_logs))
        .route("/admin/api/docs", get(handlers::get_api_docs))
        .route("/admin/api/test", post(handlers::test_api_endpoint))
        .with_state(app_state)
}