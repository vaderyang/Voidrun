use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use anyhow::Result;
use tracing::{info, warn, error};

use crate::sandbox::{SandboxManager, SandboxRequest, SandboxMode};

pub mod handlers;

/// FaaS deployment request
#[derive(Debug, Clone, Deserialize)]
pub struct DeploymentRequest {
    /// Runtime environment (bun, node, typescript)
    pub runtime: String,
    /// Main application code
    pub code: String,
    /// Additional files (optional)
    pub files: Option<Vec<FileSpec>>,
    /// Environment variables (optional)
    pub env_vars: Option<HashMap<String, String>>,
    /// Memory limit in MB (default: 256)
    pub memory_limit_mb: Option<u32>,
    /// Entry point command (optional, defaults based on runtime)
    pub entry_point: Option<String>,
    /// Auto-scale settings (optional)
    pub auto_scale: Option<AutoScaleConfig>,
    /// Whether to run as dev server with hot reload (default: true)
    pub dev_server: Option<bool>,
}

/// File specification for additional files
#[derive(Debug, Clone, Deserialize)]
pub struct FileSpec {
    /// File path relative to project root
    pub path: String,
    /// File content
    pub content: String,
    /// Whether file should be executable
    pub executable: Option<bool>,
}

/// Auto-scaling configuration
#[derive(Debug, Clone, Deserialize)]
pub struct AutoScaleConfig {
    /// Minimum number of instances (default: 0)
    pub min_instances: Option<u32>,
    /// Maximum number of instances (default: 5)
    pub max_instances: Option<u32>,
    /// Scale down after inactivity (minutes, default: 10)
    pub scale_down_after_minutes: Option<u32>,
}

/// File update request for running deployments
#[derive(Debug, Clone, Deserialize)]
pub struct FileUpdateRequest {
    /// Files to update or add
    pub files: Vec<FileSpec>,
    /// Whether to restart the dev server after update (default: true)
    pub restart_dev_server: Option<bool>,
}

/// FaaS deployment response
#[derive(Debug, Clone, Serialize)]
pub struct DeploymentResponse {
    /// Unique deployment ID
    pub deployment_id: String,
    /// Public URL to access the service
    pub url: String,
    /// Internal sandbox ID
    pub sandbox_id: String,
    /// Deployment status
    pub status: DeploymentStatus,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Runtime information
    pub runtime: String,
    /// Memory allocation
    pub memory_mb: u32,
}

/// Deployment status
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum DeploymentStatus {
    Deploying,
    Running,
    Scaling,
    Stopped,
    Failed,
}

/// Deployment information for management
#[derive(Debug, Clone)]
pub struct Deployment {
    pub id: String,
    pub sandbox_id: String,
    pub url: String,
    pub status: DeploymentStatus,
    pub created_at: DateTime<Utc>,
    pub last_accessed: Arc<RwLock<DateTime<Utc>>>,
    pub runtime: String,
    pub memory_mb: u32,
    pub auto_scale: AutoScaleConfig,
    pub request: DeploymentRequest,
}

/// FaaS Manager - handles serverless deployments
pub struct FaasManager {
    deployments: Arc<RwLock<HashMap<String, Deployment>>>,
    sandbox_manager: Arc<RwLock<SandboxManager>>,
    base_url: String,
}

impl FaasManager {
    pub fn new(sandbox_manager: Arc<RwLock<SandboxManager>>, base_url: String) -> Self {
        Self {
            deployments: Arc::new(RwLock::new(HashMap::new())),
            sandbox_manager,
            base_url,
        }
    }

    /// Deploy a new serverless function
    pub async fn deploy(&self, request: DeploymentRequest) -> Result<DeploymentResponse> {
        let deployment_id = Uuid::new_v4().to_string();
        let sandbox_id = Uuid::new_v4().to_string();
        
        info!("Starting deployment {} with runtime {}", deployment_id, request.runtime);
        info!("Deploy config - Memory: {}MB, Dev server: {}, Install deps: {}", 
              request.memory_limit_mb.unwrap_or(256),
              request.dev_server.unwrap_or(true),
              true);
        
        if let Some(ref files) = request.files {
            info!("Additional files to deploy: {}", files.len());
            for file in files {
                info!("  - {} (executable: {})", file.path, file.executable.unwrap_or(false));
            }
        }
        
        if let Some(ref env_vars) = request.env_vars {
            info!("Environment variables: {} configured", env_vars.len());
        }

        // Generate unique URL
        let url = format!("{}/faas/{}", self.base_url, deployment_id);

        // Prepare sandbox request
        info!("Creating sandbox request for deployment {}", deployment_id);
        let sandbox_request = match self.create_sandbox_request(&sandbox_id, &request).await {
            Ok(req) => {
                info!("Sandbox request created - Entry point: {}, Mode: {:?}", 
                      req.entry_point.as_ref().unwrap_or(&"default".to_string()),
                      req.mode.as_ref().unwrap_or(&SandboxMode::Persistent));
                req
            }
            Err(e) => {
                error!("Failed to create sandbox request for deployment {}: {}", deployment_id, e);
                return Err(anyhow::anyhow!("Failed to create sandbox request: {}", e));
            }
        };

        // Create sandbox
        info!("Creating sandbox {} for deployment {}", sandbox_id, deployment_id);
        let mut manager = self.sandbox_manager.write().await;
        let sandbox = match manager.create_sandbox(sandbox_request).await {
            Ok(sb) => {
                info!("Sandbox {} created successfully", sandbox_id);
                sb
            }
            Err(e) => {
                error!("Failed to create sandbox {} for deployment {}: {}", sandbox_id, deployment_id, e);
                return Err(anyhow::anyhow!("Failed to create sandbox: {}", e));
            }
        };
        drop(manager);

        // Execute initial setup
        info!("Setting up deployment {} in sandbox {}", deployment_id, sandbox_id);
        if let Err(e) = self.setup_deployment(&sandbox_id, &request).await {
            error!("Failed to setup deployment {} in sandbox {}: {}", deployment_id, sandbox_id, e);
            // Try to cleanup the sandbox
            let mut manager = self.sandbox_manager.write().await;
            if let Err(cleanup_err) = manager.delete_sandbox(&sandbox_id).await {
                error!("Failed to cleanup sandbox {} after setup failure: {}", sandbox_id, cleanup_err);
            }
            return Err(e);
        }

        // Create deployment record
        let auto_scale = request.auto_scale.clone().unwrap_or(AutoScaleConfig {
            min_instances: Some(0),
            max_instances: Some(5),
            scale_down_after_minutes: Some(10),
        });

        let deployment = Deployment {
            id: deployment_id.clone(),
            sandbox_id: sandbox_id.clone(),
            url: url.clone(),
            status: DeploymentStatus::Running,
            created_at: Utc::now(),
            last_accessed: Arc::new(RwLock::new(Utc::now())),
            runtime: request.runtime.clone(),
            memory_mb: request.memory_limit_mb.unwrap_or(256),
            auto_scale,
            request: request.clone(),
        };

        // Store deployment
        {
            let mut deployments = self.deployments.write().await;
            deployments.insert(deployment_id.clone(), deployment.clone());
            info!("Deployment {} stored in registry. Total deployments: {}", deployment_id, deployments.len());
        }

        info!("Deployment {} created successfully at {}", deployment_id, url);
        info!("Deployment summary - ID: {}, Sandbox: {}, Runtime: {}, Memory: {}MB, Status: {:?}",
              deployment_id, sandbox_id, request.runtime, request.memory_limit_mb.unwrap_or(256),
              DeploymentStatus::Running);

        Ok(DeploymentResponse {
            deployment_id: deployment_id.clone(),
            url,
            sandbox_id,
            status: DeploymentStatus::Running,
            created_at: Utc::now(),
            runtime: request.runtime,
            memory_mb: request.memory_limit_mb.unwrap_or(256),
        })
    }

    /// Get deployment information
    pub async fn get_deployment(&self, deployment_id: &str) -> Option<DeploymentResponse> {
        let deployments = self.deployments.read().await;
        if let Some(deployment) = deployments.get(deployment_id) {
            // Update last accessed time
            {
                let mut last_accessed = deployment.last_accessed.write().await;
                *last_accessed = Utc::now();
            }

            Some(DeploymentResponse {
                deployment_id: deployment.id.clone(),
                url: deployment.url.clone(),
                sandbox_id: deployment.sandbox_id.clone(),
                status: deployment.status.clone(),
                created_at: deployment.created_at,
                runtime: deployment.runtime.clone(),
                memory_mb: deployment.memory_mb,
            })
        } else {
            None
        }
    }

    /// List all deployments
    pub async fn list_deployments(&self) -> Vec<DeploymentResponse> {
        let deployments = self.deployments.read().await;
        deployments.values().map(|d| DeploymentResponse {
            deployment_id: d.id.clone(),
            url: d.url.clone(),
            sandbox_id: d.sandbox_id.clone(),
            status: d.status.clone(),
            created_at: d.created_at,
            runtime: d.runtime.clone(),
            memory_mb: d.memory_mb,
        }).collect()
    }

    /// Stop and remove a deployment
    pub async fn undeploy(&self, deployment_id: &str) -> Result<()> {
        info!("Starting undeploy for deployment {}", deployment_id);
        
        let deployment = {
            let mut deployments = self.deployments.write().await;
            let total_before = deployments.len();
            let deployment = deployments.remove(deployment_id);
            
            if deployment.is_some() {
                info!("Removed deployment {} from registry. Deployments: {} -> {}", 
                      deployment_id, total_before, deployments.len());
            } else {
                error!("Deployment {} not found in registry (total: {})", deployment_id, total_before);
            }
            
            deployment
        };

        if let Some(deployment) = deployment {
            info!("Undeploying {} - Sandbox: {}, Runtime: {}, Created: {}", 
                  deployment_id, deployment.sandbox_id, deployment.runtime, deployment.created_at);
            
            // Calculate deployment lifetime
            let lifetime = Utc::now() - deployment.created_at;
            info!("Deployment {} was active for {} minutes", deployment_id, lifetime.num_minutes());
            
            // Stop sandbox
            info!("Deleting sandbox {} for deployment {}", deployment.sandbox_id, deployment_id);
            let mut manager = self.sandbox_manager.write().await;
            match manager.delete_sandbox(&deployment.sandbox_id).await {
                Ok(()) => {
                    info!("Sandbox {} deleted successfully", deployment.sandbox_id);
                }
                Err(e) => {
                    error!("Failed to delete sandbox {} for deployment {}: {}", 
                          deployment.sandbox_id, deployment_id, e);
                    warn!("Deployment {} removed from registry but sandbox {} cleanup failed", 
                          deployment_id, deployment.sandbox_id);
                    // Don't return error here - deployment is already removed from registry
                }
            }
            
            info!("Deployment {} undeployed successfully", deployment_id);
            Ok(())
        } else {
            error!("Cannot undeploy - Deployment {} not found", deployment_id);
            Err(anyhow::anyhow!("Deployment {} not found", deployment_id))
        }
    }

    /// Get deployment by ID for proxying
    pub async fn get_deployment_for_proxy(&self, deployment_id: &str) -> Option<String> {
        let deployments = self.deployments.read().await;
        if let Some(deployment) = deployments.get(deployment_id) {
            // Update last accessed time
            tokio::spawn({
                let last_accessed = deployment.last_accessed.clone();
                async move {
                    let mut last_accessed = last_accessed.write().await;
                    *last_accessed = Utc::now();
                }
            });
            
            Some(deployment.sandbox_id.clone())
        } else {
            None
        }
    }

    /// Update files in a running deployment
    pub async fn update_files(&self, deployment_id: &str, update_request: FileUpdateRequest) -> Result<()> {
        info!("Starting file update for deployment {}", deployment_id);
        info!("Update request - Files: {}, Restart dev server: {}", 
              update_request.files.len(),
              update_request.restart_dev_server.unwrap_or(true));
        
        let deployment = {
            let deployments = self.deployments.read().await;
            match deployments.get(deployment_id).cloned() {
                Some(d) => {
                    info!("Found deployment {} - Sandbox: {}, Status: {:?}", 
                          deployment_id, d.sandbox_id, d.status);
                    Some(d)
                }
                None => {
                    error!("Deployment {} not found in registry", deployment_id);
                    None
                }
            }
        };

        if let Some(deployment) = deployment {
            info!("Updating {} files for deployment {} in sandbox {}", 
                  update_request.files.len(), deployment_id, deployment.sandbox_id);
            
            let mut manager = self.sandbox_manager.write().await;
            
            // Update files in the container
            for file in &update_request.files {
                info!("Adding file {} to sandbox {} (executable: {})", 
                      file.path, deployment.sandbox_id, file.executable.unwrap_or(false));
                
                if let Err(e) = manager.add_files_to_sandbox(&deployment.sandbox_id, vec![crate::sandbox::SandboxFile {
                    path: file.path.clone(),
                    content: file.content.clone(),
                    is_executable: file.executable,
                }]).await {
                    error!("Failed to add file {} to sandbox {}: {}", file.path, deployment.sandbox_id, e);
                    warn!("Continuing with remaining files despite error");
                }
            }

            // Update files directly in the running container
            info!("Updating files directly in running container {}", deployment.sandbox_id);
            if let Err(e) = self.update_container_files(&deployment.sandbox_id, &update_request.files).await {
                error!("Failed to update container files for sandbox {}: {}", deployment.sandbox_id, e);
                return Err(anyhow::anyhow!("Failed to update container files: {}", e));
            }
            info!("Container files updated successfully");

            // Restart dev server if requested (default: true)
            let should_restart = update_request.restart_dev_server.unwrap_or(true);
            let is_dev_server = deployment.request.dev_server.unwrap_or(false);
            
            if should_restart && is_dev_server {
                info!("Restarting dev server for deployment {} in sandbox {}", 
                      deployment_id, deployment.sandbox_id);
                if let Err(e) = self.restart_dev_server(&deployment.sandbox_id, &deployment.request).await {
                    error!("Failed to restart dev server for sandbox {}: {}", deployment.sandbox_id, e);
                    return Err(anyhow::anyhow!("Failed to restart dev server: {}", e));
                }
                info!("Dev server restarted successfully");
            } else {
                info!("Skipping dev server restart - Requested: {}, Is dev server: {}", 
                      should_restart, is_dev_server);
            }

            // Update last accessed time
            {
                let mut last_accessed = deployment.last_accessed.write().await;
                *last_accessed = Utc::now();
            }

            info!("File update completed successfully for deployment {}", deployment_id);
            info!("Update summary - Deployment: {}, Files updated: {}, Dev server restarted: {}",
                  deployment_id, update_request.files.len(), 
                  should_restart && is_dev_server);
            Ok(())
        } else {
            error!("Cannot update files - Deployment {} not found", deployment_id);
            Err(anyhow::anyhow!("Deployment {} not found", deployment_id))
        }
    }

    /// Start cleanup task for idle deployments
    pub async fn start_cleanup_task(&self) {
        let deployments = self.deployments.clone();
        let sandbox_manager = self.sandbox_manager.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60)); // Check every minute
            
            loop {
                interval.tick().await;
                
                let now = Utc::now();
                let mut to_remove = Vec::new();
                
                {
                    let deployments_read = deployments.read().await;
                    for (id, deployment) in deployments_read.iter() {
                        let last_accessed = *deployment.last_accessed.read().await;
                        let idle_minutes = (now - last_accessed).num_minutes();
                        let scale_down_after = deployment.auto_scale.scale_down_after_minutes.unwrap_or(10) as i64;
                        
                        if idle_minutes > scale_down_after {
                            to_remove.push((id.clone(), deployment.sandbox_id.clone()));
                        }
                    }
                }
                
                // Remove idle deployments
                if !to_remove.is_empty() {
                    info!("Auto-cleanup: Found {} idle deployments to remove", to_remove.len());
                }
                
                for (deployment_id, sandbox_id) in to_remove {
                    info!("Auto-cleanup: Removing idle deployment {} (sandbox: {})", deployment_id, sandbox_id);
                    
                    {
                        let mut deployments_write = deployments.write().await;
                        if let Some(deployment) = deployments_write.get(&deployment_id) {
                            let idle_time = (now - *deployment.last_accessed.read().await).num_minutes();
                            info!("Auto-cleanup: Deployment {} was idle for {} minutes", deployment_id, idle_time);
                        }
                        deployments_write.remove(&deployment_id);
                    }
                    
                    // Stop sandbox
                    info!("Auto-cleanup: Deleting sandbox {} for deployment {}", sandbox_id, deployment_id);
                    let mut manager = sandbox_manager.write().await;
                    match manager.delete_sandbox(&sandbox_id).await {
                        Ok(()) => {
                            info!("Auto-cleanup: Successfully deleted sandbox {} for deployment {}", 
                                  sandbox_id, deployment_id);
                        }
                        Err(e) => {
                            error!("Auto-cleanup: Failed to delete sandbox {} for deployment {}: {}", 
                                   sandbox_id, deployment_id, e);
                        }
                    }
                }
            }
        });
    }

    /// Create sandbox request from deployment request
    async fn create_sandbox_request(&self, sandbox_id: &str, request: &DeploymentRequest) -> Result<SandboxRequest> {
        // Convert files
        let files = if let Some(ref file_specs) = request.files {
            Some(file_specs.iter().map(|f| crate::sandbox::SandboxFile {
                path: f.path.clone(),
                content: f.content.clone(),
                is_executable: f.executable,
            }).collect())
        } else {
            None
        };

        // Determine entry point based on runtime
        let entry_point = request.entry_point.clone().unwrap_or_else(|| {
            match request.runtime.as_str() {
                "bun" => "bun dev".to_string(),
                "node" | "nodejs" => "npm run dev".to_string(),
                "typescript" | "ts" => "bun dev".to_string(),
                _ => "npm run dev".to_string(),
            }
        });

        Ok(SandboxRequest {
            id: sandbox_id.to_string(),
            runtime: request.runtime.clone(),
            code: request.code.clone(),
            entry_point: Some(entry_point),
            files,
            env_vars: request.env_vars.clone().unwrap_or_default(),
            timeout_ms: 300000, // 5 minutes default
            memory_limit_mb: request.memory_limit_mb.unwrap_or(256) as u64,
            mode: Some(SandboxMode::Persistent),
            dev_server: Some(true),
            install_deps: Some(true),
        })
    }

    /// Setup deployment after sandbox creation
    async fn setup_deployment(&self, sandbox_id: &str, request: &DeploymentRequest) -> Result<()> {
        info!("Starting deployment setup for sandbox {}", sandbox_id);
        info!("Executing entry point: {}", request.entry_point.as_ref()
              .unwrap_or(&match request.runtime.as_str() {
                  "bun" => "bun dev".to_string(),
                  "node" | "nodejs" => "npm run dev".to_string(),
                  _ => "npm run dev".to_string(),
              }));
        
        // Execute the sandbox to start the web service
        let mut manager = self.sandbox_manager.write().await;
        
        // For FaaS, we execute the sandbox to start the service
        info!("Executing sandbox {} to start web service", sandbox_id);
        let exec_result = match manager.execute_sandbox(sandbox_id).await {
            Ok(result) => {
                info!("Sandbox execution completed - Success: {}, Exit code: {:?}", 
                      result.success, result.exit_code);
                if !result.stdout.is_empty() {
                    info!("Sandbox stdout: {}", result.stdout);
                }
                if !result.stderr.is_empty() {
                    warn!("Sandbox stderr: {}", result.stderr);
                }
                result
            }
            Err(e) => {
                error!("Failed to execute sandbox {}: {}", sandbox_id, e);
                return Err(anyhow::anyhow!("Failed to execute deployment setup: {}", e));
            }
        };

        if !exec_result.success {
            error!("Deployment setup failed for sandbox {} - Exit code: {:?}, Error: {}", 
                   sandbox_id, exec_result.exit_code, exec_result.stderr);
            return Err(anyhow::anyhow!("Deployment setup failed: {}", exec_result.stderr));
        }

        info!("Deployment setup completed successfully for sandbox {}", sandbox_id);
        Ok(())
    }

    /// Update files using the sandbox backend abstraction
    async fn update_container_files(&self, sandbox_id: &str, files: &[FileSpec]) -> Result<()> {
        info!("Converting {} FileSpec to SandboxFile format", files.len());
        
        // Convert FileSpec to SandboxFile
        let sandbox_files: Vec<crate::sandbox::SandboxFile> = files.iter().map(|f| {
            info!("  - Converting file: {} (size: {} bytes)", f.path, f.content.len());
            crate::sandbox::SandboxFile {
                path: f.path.clone(),
                content: f.content.clone(),
                is_executable: f.executable,
            }
        }).collect();
        
        // Use sandbox manager to get the backend and call update_files
        info!("Getting sandbox backend for file updates");
        let manager = self.sandbox_manager.read().await;
        if let Some(backend) = manager.get_backend() {
            info!("Calling backend.update_files for sandbox {}", sandbox_id);
            match backend.update_files(sandbox_id, &sandbox_files).await {
                Ok(()) => {
                    info!("Backend update_files completed successfully");
                    Ok(())
                }
                Err(e) => {
                    error!("Backend update_files failed for sandbox {}: {}", sandbox_id, e);
                    Err(e)
                }
            }
        } else {
            error!("No sandbox backend available for file updates");
            return Err(anyhow::anyhow!("No sandbox backend available"));
        }
    }

    /// Restart the development server using sandbox backend abstraction
    async fn restart_dev_server(&self, sandbox_id: &str, request: &DeploymentRequest) -> Result<()> {
        // Determine the command to run
        let command = if let Some(entry_point) = &request.entry_point {
            info!("Using custom entry point: {}", entry_point);
            entry_point.clone()
        } else {
            let default_cmd = match request.runtime.as_str() {
                "bun" => "bun dev".to_string(),
                "node" | "nodejs" => "npm run dev".to_string(),
                _ => "bun dev".to_string(),
            };
            info!("Using default entry point for runtime {}: {}", request.runtime, default_cmd);
            default_cmd
        };
        
        info!("Restarting process in sandbox {} with command: {}", sandbox_id, command);
        
        // Use sandbox manager to get the backend and call restart_process
        let manager = self.sandbox_manager.read().await;
        if let Some(backend) = manager.get_backend() {
            info!("Calling backend.restart_process for sandbox {}", sandbox_id);
            match backend.restart_process(sandbox_id, &command).await {
                Ok(()) => {
                    info!("Backend restart_process completed successfully for sandbox {}", sandbox_id);
                    Ok(())
                }
                Err(e) => {
                    error!("Backend restart_process failed for sandbox {}: {}", sandbox_id, e);
                    Err(e)
                }
            }
        } else {
            error!("No sandbox backend available for process restart");
            return Err(anyhow::anyhow!("No sandbox backend available"));
        }
    }
}