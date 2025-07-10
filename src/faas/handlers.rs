use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, delete, put},
    Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error, warn};

use super::{FaasManager, DeploymentRequest, DeploymentResponse, FileUpdateRequest};
use crate::sandbox::SandboxManager;

/// FaaS API state
#[derive(Clone)]
pub struct FaasState {
    pub faas_manager: Arc<FaasManager>,
}

impl FaasState {
    pub fn new(sandbox_manager: Arc<RwLock<SandboxManager>>, base_url: String) -> Self {
        Self {
            faas_manager: Arc::new(FaasManager::new(sandbox_manager, base_url)),
        }
    }
}

/// Deploy a new serverless function
///
/// POST /faas/deploy
/// Body: DeploymentRequest
/// Returns: DeploymentResponse with unique URL
pub async fn deploy_function(
    State(state): State<FaasState>,
    Json(request): Json<DeploymentRequest>,
) -> Result<Json<DeploymentResponse>, StatusCode> {
    info!("[HTTP] Deploy request received - Runtime: {}, Memory: {}MB, Dev server: {}", 
          request.runtime, 
          request.memory_limit_mb.unwrap_or(256),
          request.dev_server.unwrap_or(true));
    
    if let Some(ref files) = request.files {
        info!("[HTTP] Deploy includes {} additional files", files.len());
    }
    
    if let Some(ref env_vars) = request.env_vars {
        info!("[HTTP] Deploy includes {} environment variables", env_vars.len());
    }
    
    match state.faas_manager.deploy(request).await {
        Ok(response) => {
            info!("[HTTP] Function deployed successfully - ID: {}, URL: {}, Sandbox: {}", 
                  response.deployment_id, response.url, response.sandbox_id);
            Ok(Json(response))
        }
        Err(e) => {
            error!("[HTTP] Failed to deploy function: {}", e);
            error!("[HTTP] Deploy error details: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get deployment information
///
/// GET /faas/deployments/{deployment_id}
/// Returns: DeploymentResponse
pub async fn get_deployment(
    State(state): State<FaasState>,
    Path(deployment_id): Path<String>,
) -> Result<Json<DeploymentResponse>, StatusCode> {
    match state.faas_manager.get_deployment(&deployment_id).await {
        Some(deployment) => Ok(Json(deployment)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// List all deployments
///
/// GET /faas/deployments
/// Returns: Vec<DeploymentResponse>
pub async fn list_deployments(
    State(state): State<FaasState>,
) -> Result<Json<Vec<DeploymentResponse>>, StatusCode> {
    let deployments = state.faas_manager.list_deployments().await;
    Ok(Json(deployments))
}

/// Undeploy a function
///
/// DELETE /faas/deployments/{deployment_id}
/// Returns: 204 No Content on success
pub async fn undeploy_function(
    State(state): State<FaasState>,
    Path(deployment_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    info!("[HTTP] Undeploy request received for deployment: {}", deployment_id);
    
    // Check if deployment exists first
    let deployment_info = state.faas_manager.get_deployment(&deployment_id).await;
    if let Some(info) = deployment_info {
        info!("[HTTP] Found deployment {} - Sandbox: {}, Status: {:?}", 
              deployment_id, info.sandbox_id, info.status);
    } else {
        warn!("[HTTP] Undeploy requested for non-existent deployment: {}", deployment_id);
    }
    
    match state.faas_manager.undeploy(&deployment_id).await {
        Ok(()) => {
            info!("[HTTP] Function undeployed successfully: {}", deployment_id);
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            error!("[HTTP] Failed to undeploy function {}: {}", deployment_id, e);
            error!("[HTTP] Undeploy error details: {:?}", e);
            
            if e.to_string().contains("not found") {
                error!("[HTTP] Deployment {} not found for undeploy", deployment_id);
                Err(StatusCode::NOT_FOUND)
            } else {
                error!("[HTTP] Internal error during undeploy");
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// Update files in a running deployment
///
/// PUT /faas/deployments/{deployment_id}/files
/// Body: FileUpdateRequest
/// Returns: 200 OK on success
pub async fn update_files(
    State(state): State<FaasState>,
    Path(deployment_id): Path<String>,
    Json(request): Json<FileUpdateRequest>,
) -> Result<StatusCode, StatusCode> {
    info!("[HTTP] Update files request for deployment: {}", deployment_id);
    info!("[HTTP] Update details - Files: {}, Restart dev server: {}", 
          request.files.len(),
          request.restart_dev_server.unwrap_or(true));
    
    for (idx, file) in request.files.iter().enumerate() {
        info!("[HTTP] File {} - Path: {}, Size: {} bytes, Executable: {}", 
              idx + 1, file.path, file.content.len(), file.executable.unwrap_or(false));
    }
    
    match state.faas_manager.update_files(&deployment_id, request).await {
        Ok(()) => {
            info!("[HTTP] Files updated successfully for deployment: {}", deployment_id);
            Ok(StatusCode::OK)
        }
        Err(e) => {
            error!("[HTTP] Failed to update files for deployment {}: {}", deployment_id, e);
            error!("[HTTP] Update error details: {:?}", e);
            if e.to_string().contains("not found") {
                error!("[HTTP] Deployment {} not found", deployment_id);
                Err(StatusCode::NOT_FOUND)
            } else {
                error!("[HTTP] Internal error during update");
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// Create FaaS router
pub fn create_faas_router(state: FaasState) -> Router {
    Router::new()
        .route("/faas/deploy", post(deploy_function))
        .route("/faas/deployments", get(list_deployments))
        .route("/faas/deployments/:deployment_id", get(get_deployment))
        .route("/faas/deployments/:deployment_id", delete(undeploy_function))
        .route("/faas/deployments/:deployment_id/files", put(update_files))
        .with_state(state)
}