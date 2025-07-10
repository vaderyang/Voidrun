use anyhow::Result;
use std::collections::HashMap;

use super::{Sandbox, SandboxRequest, SandboxResponse, SandboxStatus, SandboxFile};
use super::backend::{create_backend, SandboxBackend, SandboxBackendType};
use crate::api::SandboxInfo;

pub struct SandboxManager {
    sandboxes: HashMap<String, Sandbox>,
    backend: Box<dyn SandboxBackend>,
    backend_type: SandboxBackendType,
}

impl SandboxManager {
    pub async fn new(backend_type: SandboxBackendType) -> Result<Self> {
        let backend = create_backend(backend_type.clone())?;
        
        if !backend.is_available().await {
            anyhow::bail!("Selected backend {:?} is not available", backend_type);
        }

        Ok(Self {
            sandboxes: HashMap::new(),
            backend,
            backend_type,
        })
    }

    pub async fn create_sandbox(&mut self, request: SandboxRequest) -> Result<()> {
        let sandbox = Sandbox::new(request.clone(), self.backend_type.clone());
        
        self.backend.create_sandbox(&request).await?;
        
        self.sandboxes.insert(request.id.clone(), sandbox);
        Ok(())
    }

    pub async fn execute_sandbox(&mut self, sandbox_id: &str) -> Result<SandboxResponse> {
        let sandbox = self.sandboxes.get_mut(sandbox_id)
            .ok_or_else(|| anyhow::anyhow!("Sandbox {} not found", sandbox_id))?;

        sandbox.status = SandboxStatus::Running;
        
        let response = self.backend.execute_sandbox(&sandbox.request).await?;
        
        sandbox.status = if response.success {
            SandboxStatus::Completed
        } else {
            SandboxStatus::Failed
        };

        Ok(response)
    }

    pub async fn execute_sandbox_direct(&mut self, request: SandboxRequest) -> Result<SandboxResponse> {
        // For one-shot execution, just execute directly without storing the sandbox
        self.backend.execute_sandbox(&request).await
    }

    pub async fn delete_sandbox(&mut self, sandbox_id: &str) -> Result<()> {
        let _sandbox = self.sandboxes.remove(sandbox_id)
            .ok_or_else(|| anyhow::anyhow!("Sandbox {} not found", sandbox_id))?;

        self.backend.cleanup_sandbox(sandbox_id).await?;
        Ok(())
    }

    pub async fn get_sandbox_info(&self, sandbox_id: &str) -> Option<SandboxInfo> {
        self.sandboxes.get(sandbox_id).map(|s| s.to_info())
    }

    pub async fn list_sandboxes(&self) -> Vec<SandboxInfo> {
        self.sandboxes.values().map(|s| s.to_info()).collect()
    }
    
    pub async fn get_all_sandboxes(&self) -> Vec<&Sandbox> {
        self.sandboxes.values().collect()
    }
    
    pub fn get_backend_type(&self) -> &SandboxBackendType {
        &self.backend_type
    }
    
    pub fn get_backend(&self) -> Option<&dyn SandboxBackend> {
        Some(self.backend.as_ref())
    }

    pub async fn cleanup_all(&mut self) -> Result<()> {
        let sandbox_ids: Vec<String> = self.sandboxes.keys().cloned().collect();
        
        for id in sandbox_ids {
            if let Err(e) = self.delete_sandbox(&id).await {
                tracing::warn!("Failed to cleanup sandbox {}: {}", id, e);
            }
        }
        
        Ok(())
    }

    pub async fn add_files_to_sandbox(&mut self, sandbox_id: &str, files: Vec<SandboxFile>) -> Result<()> {
        let sandbox = self.sandboxes.get_mut(sandbox_id)
            .ok_or_else(|| anyhow::anyhow!("Sandbox {} not found", sandbox_id))?;

        // Add files to the sandbox request
        if let Some(ref mut existing_files) = sandbox.request.files {
            existing_files.extend(files);
        } else {
            sandbox.request.files = Some(files);
        }

        Ok(())
    }
}