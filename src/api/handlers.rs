use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

use super::{AppState, CreateSandboxRequest, ExecutionResult, SandboxInfo, SandboxFile};
use crate::sandbox::SandboxRequest;

pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "sandbox-service",
        "version": "0.1.0"
    }))
}

pub async fn execute_one_shot(
    State(state): State<AppState>,
    Json(req): Json<CreateSandboxRequest>,
) -> Result<Json<Value>, StatusCode> {
    let sandbox_id = Uuid::new_v4().to_string();
    
    let sandbox_req = SandboxRequest {
        id: sandbox_id.clone(),
        runtime: req.runtime.clone(),
        code: req.code,
        entry_point: req.entry_point,
        timeout_ms: req.timeout_ms.unwrap_or(30000),
        memory_limit_mb: req.memory_limit_mb.unwrap_or(512),
        env_vars: req.env_vars.unwrap_or_default(),
        files: req.files.map(|files| files.into_iter().map(|f| crate::sandbox::SandboxFile {
            path: f.path,
            content: f.content,
            is_executable: f.is_executable,
        }).collect()),
        mode: Some(crate::sandbox::SandboxMode::OneShot),
        install_deps: req.install_deps,
        dev_server: req.dev_server,
    };

    let mut manager = state.write().await;
    match manager.execute_sandbox_direct(sandbox_req).await {
        Ok(result) => {
            Ok(Json(json!({
                "success": result.success,
                "stdout": result.stdout,
                "stderr": result.stderr,
                "exit_code": result.exit_code,
                "execution_time_ms": result.execution_time_ms,
                "is_running": result.is_running,
                "dev_server_url": result.dev_server_url
            })))
        }
        Err(e) => {
            Ok(Json(json!({
                "success": false,
                "stdout": "",
                "stderr": format!("Execution failed: {}", e),
                "exit_code": Some(1),
                "execution_time_ms": 0,
                "is_running": Some(false),
                "dev_server_url": None::<String>
            })))
        }
    }
}

pub async fn create_sandbox(
    State(state): State<AppState>,
    Json(req): Json<CreateSandboxRequest>,
) -> Result<Json<SandboxInfo>, StatusCode> {
    let sandbox_id = Uuid::new_v4().to_string();
    
    let sandbox_req = SandboxRequest {
        id: sandbox_id.clone(),
        runtime: req.runtime.clone(),
        code: req.code,
        entry_point: req.entry_point,
        timeout_ms: req.timeout_ms.unwrap_or(30000),
        memory_limit_mb: req.memory_limit_mb.unwrap_or(512),
        env_vars: req.env_vars.unwrap_or_default(),
        files: req.files.map(|files| files.into_iter().map(|f| crate::sandbox::SandboxFile {
            path: f.path,
            content: f.content,
            is_executable: f.is_executable,
        }).collect()),
        mode: req.mode.as_deref().map(|m| match m {
            "persistent" => crate::sandbox::SandboxMode::Persistent,
            _ => crate::sandbox::SandboxMode::OneShot,
        }),
        install_deps: req.install_deps,
        dev_server: req.dev_server,
    };

    let mut manager = state.write().await;
    match manager.create_sandbox(sandbox_req).await {
        Ok(_) => {
            let info = SandboxInfo {
                id: sandbox_id,
                status: "created".to_string(),
                runtime: req.runtime,
                created_at: chrono::Utc::now().to_rfc3339(),
                timeout_ms: req.timeout_ms.unwrap_or(30000),
                memory_limit_mb: req.memory_limit_mb.unwrap_or(512),
            };
            Ok(Json(info))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_sandbox(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SandboxInfo>, StatusCode> {
    let manager = state.read().await;
    match manager.get_sandbox_info(&id).await {
        Some(info) => Ok(Json(info)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn delete_sandbox(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let mut manager = state.write().await;
    match manager.delete_sandbox(&id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn execute_code(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ExecutionResult>, StatusCode> {
    let mut manager = state.write().await;
    match manager.execute_sandbox(&id).await {
        Ok(result) => {
            let exec_result = ExecutionResult {
                sandbox_id: id,
                success: result.success,
                stdout: result.stdout,
                stderr: result.stderr,
                exit_code: result.exit_code,
                execution_time_ms: result.execution_time_ms,
            };
            Ok(Json(exec_result))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn list_sandboxes(
    State(state): State<AppState>,
) -> Result<Json<Vec<SandboxInfo>>, StatusCode> {
    let manager = state.read().await;
    let sandboxes = manager.list_sandboxes().await;
    Ok(Json(sandboxes))
}

pub async fn upload_files(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(files): Json<Vec<SandboxFile>>,
) -> Result<Json<Value>, StatusCode> {
    let mut manager = state.write().await;
    
    // Convert API files to sandbox files
    let sandbox_files: Vec<crate::sandbox::SandboxFile> = files.into_iter().map(|f| {
        crate::sandbox::SandboxFile {
            path: f.path,
            content: f.content,
            is_executable: f.is_executable,
        }
    }).collect();
    
    match manager.add_files_to_sandbox(&id, sandbox_files).await {
        Ok(_) => Ok(Json(json!({
            "message": "Files uploaded successfully",
            "sandbox_id": id
        }))),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}