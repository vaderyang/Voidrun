use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, delete, put},
    Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};

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
    info!("Deploying new function with runtime: {}", request.runtime);
    
    match state.faas_manager.deploy(request).await {
        Ok(response) => {
            info!("Function deployed successfully: {}", response.deployment_id);
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to deploy function: {}", e);
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
    info!("Undeploying function: {}", deployment_id);
    
    match state.faas_manager.undeploy(&deployment_id).await {
        Ok(()) => {
            info!("Function undeployed successfully: {}", deployment_id);
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            error!("Failed to undeploy function {}: {}", deployment_id, e);
            if e.to_string().contains("not found") {
                Err(StatusCode::NOT_FOUND)
            } else {
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
    info!("Updating files for deployment: {}", deployment_id);
    
    match state.faas_manager.update_files(&deployment_id, request).await {
        Ok(()) => {
            info!("Files updated successfully for deployment: {}", deployment_id);
            Ok(StatusCode::OK)
        }
        Err(e) => {
            error!("Failed to update files for deployment {}: {}", deployment_id, e);
            if e.to_string().contains("not found") {
                Err(StatusCode::NOT_FOUND)
            } else {
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