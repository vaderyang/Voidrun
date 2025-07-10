use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{SandboxRequest, SandboxResponse};

pub mod docker;
pub mod nsjail;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SandboxBackendType {
    Docker,
    Nsjail,
    #[cfg(feature = "firecracker")]
    Firecracker,
    #[cfg(feature = "gvisor")]
    Gvisor,
}

#[async_trait]
pub trait SandboxBackend: Send + Sync {
    async fn create_sandbox(&self, request: &SandboxRequest) -> Result<()>;
    async fn execute_sandbox(&self, request: &SandboxRequest) -> Result<SandboxResponse>;
    async fn cleanup_sandbox(&self, sandbox_id: &str) -> Result<()>;
    async fn is_available(&self) -> bool;
    fn backend_type(&self) -> SandboxBackendType;
}

pub fn create_backend(backend_type: SandboxBackendType) -> Result<Box<dyn SandboxBackend>> {
    match backend_type {
        SandboxBackendType::Docker => {
            #[cfg(feature = "docker")]
            {
                Ok(Box::new(docker::DockerBackend::new()?))
            }
            #[cfg(not(feature = "docker"))]
            {
                anyhow::bail!("Docker backend not available. Enable 'docker' feature.")
            }
        }
        SandboxBackendType::Nsjail => {
            Ok(Box::new(nsjail::NsjailBackend::new()?))
        }
        #[cfg(feature = "firecracker")]
        SandboxBackendType::Firecracker => {
            anyhow::bail!("Firecracker backend not yet implemented")
        }
        #[cfg(feature = "gvisor")]
        SandboxBackendType::Gvisor => {
            anyhow::bail!("gVisor backend not yet implemented")
        }
    }
}