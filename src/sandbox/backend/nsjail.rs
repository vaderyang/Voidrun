use anyhow::{Context, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::process::Stdio;
use std::time::Instant;
use tempfile::TempDir;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::time::{timeout, Duration};

use super::{SandboxBackend, SandboxBackendType};
use crate::sandbox::{SandboxRequest, SandboxResponse};

pub struct NsjailBackend {
    nsjail_path: String,
    temp_dir: TempDir,
}

impl NsjailBackend {
    pub fn new() -> Result<Self> {
        let nsjail_path = which::which("nsjail")
            .context("nsjail not found in PATH. Please install nsjail.")?
            .to_string_lossy()
            .to_string();

        let temp_dir = tempfile::TempDir::new()
            .context("Failed to create temporary directory")?;

        Ok(Self {
            nsjail_path,
            temp_dir,
        })
    }

    async fn setup_sandbox_env(&self, request: &SandboxRequest) -> Result<String> {
        let sandbox_dir = self.temp_dir.path().join(&request.id);
        fs::create_dir_all(&sandbox_dir).await
            .context("Failed to create sandbox directory")?;

        let file_extension = match request.runtime.as_str() {
            "node" | "nodejs" => "js",
            "bun" => "js",
            "typescript" | "ts" => "ts",
            _ => anyhow::bail!("Unsupported runtime: {}", request.runtime),
        };

        let code_file = sandbox_dir.join(format!("index.{}", file_extension));
        fs::write(&code_file, &request.code).await
            .context("Failed to write code file")?;

        // Create additional files if provided
        if let Some(files) = &request.files {
            for file in files {
                let file_path = if file.path.starts_with('/') {
                    sandbox_dir.join(file.path.trim_start_matches('/'))
                } else {
                    sandbox_dir.join(&file.path)
                };

                // Create parent directories if they don't exist
                if let Some(parent) = file_path.parent() {
                    fs::create_dir_all(parent).await
                        .context("Failed to create parent directory")?;
                }

                fs::write(&file_path, &file.content).await
                    .context("Failed to write file")?;

                // Make executable if specified
                if file.is_executable.unwrap_or(false) {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let mut perms = fs::metadata(&file_path).await?.permissions();
                        perms.set_mode(perms.mode() | 0o755);
                        fs::set_permissions(&file_path, perms).await
                            .context("Failed to set file permissions")?;
                    }
                }
            }
        }

        Ok(sandbox_dir.to_string_lossy().to_string())
    }

    async fn execute_with_nsjail(&self, request: &SandboxRequest, sandbox_dir: &str) -> Result<SandboxResponse> {
        let start_time = Instant::now();

        let runtime_cmd = match request.runtime.as_str() {
            "node" | "nodejs" => vec!["node", "index.js"],
            "bun" => vec!["bun", "run", "index.js"],
            "typescript" | "ts" => vec!["npx", "ts-node", "index.ts"],
            _ => anyhow::bail!("Unsupported runtime: {}", request.runtime),
        };

        let mut cmd = Command::new(&self.nsjail_path);
        cmd.args([
            "--mode", "o",  // Once mode - run once and exit
            "--chroot", sandbox_dir,
            "--user", "nobody",
            "--group", "nogroup",
            "--hostname", "sandbox",
            "--cwd", "/",
            "--mount", "none,/,tmpfs,rw,size=50m",
            "--mount", format!("{},/sandbox,bind,rw", sandbox_dir).as_str(),
            "--rlimit_as", &format!("{}", request.memory_limit_mb * 1024 * 1024),
            "--rlimit_cpu", "30", // 30 seconds CPU time
            "--rlimit_fsize", "10485760", // 10MB file size limit
            "--rlimit_nofile", "64", // 64 open files
            "--disable_no_new_privs",
            "--cap", "-all",
            "--seccomp_policy", "/dev/null", // Allow all syscalls for now
            "--time_limit", &format!("{}", request.timeout_ms / 1000), // Convert to seconds
            "--really_quiet",
            "--",
        ]);

        cmd.args(runtime_cmd);
        cmd.current_dir(sandbox_dir);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdin(Stdio::null());

        // Set environment variables
        for (key, value) in &request.env_vars {
            cmd.env(key, value);
        }

        let timeout_duration = Duration::from_millis(request.timeout_ms + 1000); // Add 1s buffer
        let child_result = cmd.spawn();

        match child_result {
            Ok(mut child) => {
                let output_result = timeout(
                    Duration::from_millis(request.timeout_ms + 1000),
                    async {
                        child.wait_with_output().await
                    }
                ).await;

                let execution_time = start_time.elapsed().as_millis() as u64;

                match output_result {
                    Ok(Ok(output)) => {
                        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                        let exit_code = output.status.code();
                        let success = output.status.success();

                        Ok(SandboxResponse {
                            success,
                            stdout,
                            stderr,
                            exit_code,
                            execution_time_ms: execution_time,
                            is_running: Some(false),
                            dev_server_url: None,
                        })
                    }
                    Ok(Err(e)) => {
                        Ok(SandboxResponse {
                            success: false,
                            stdout: String::new(),
                            stderr: format!("Process error: {}", e),
                            exit_code: Some(1),
                            execution_time_ms: execution_time,
                            is_running: Some(false),
                            dev_server_url: None,
                        })
                    }
                    Err(_) => {
                        Ok(SandboxResponse {
                            success: false,
                            stdout: String::new(),
                            stderr: "Execution timed out".to_string(),
                            exit_code: Some(124),
                            execution_time_ms: execution_time,
                            is_running: Some(false),
                            dev_server_url: None,
                        })
                    }
                }
            }
            Err(e) => {
                Ok(SandboxResponse {
                    success: false,
                    stdout: String::new(),
                    stderr: format!("Failed to spawn process: {}", e),
                    exit_code: Some(1),
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    is_running: Some(false),
                    dev_server_url: None,
                })
            }
        }
    }
}

#[async_trait]
impl SandboxBackend for NsjailBackend {
    async fn create_sandbox(&self, request: &SandboxRequest) -> Result<()> {
        self.setup_sandbox_env(request).await?;
        Ok(())
    }

    async fn execute_sandbox(&self, request: &SandboxRequest) -> Result<SandboxResponse> {
        let sandbox_dir = self.setup_sandbox_env(request).await?;
        let response = self.execute_with_nsjail(request, &sandbox_dir).await?;
        Ok(response)
    }

    async fn cleanup_sandbox(&self, sandbox_id: &str) -> Result<()> {
        let sandbox_dir = self.temp_dir.path().join(sandbox_id);
        if sandbox_dir.exists() {
            fs::remove_dir_all(sandbox_dir).await
                .context("Failed to cleanup sandbox directory")?;
        }
        Ok(())
    }

    async fn is_available(&self) -> bool {
        Command::new(&self.nsjail_path)
            .arg("--help")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|status| status.success())
            .unwrap_or(false)
    }

    fn backend_type(&self) -> SandboxBackendType {
        SandboxBackendType::Nsjail
    }
}