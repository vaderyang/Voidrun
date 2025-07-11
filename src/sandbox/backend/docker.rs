#[cfg(feature = "docker")]
use anyhow::{Context, Result};
use async_trait::async_trait;
use bollard::{
    container::{Config, CreateContainerOptions, RemoveContainerOptions, StartContainerOptions},
    exec::{CreateExecOptions, StartExecResults},
    image::CreateImageOptions,
    ClientVersion, Docker,
};
use futures_util::StreamExt;
use std::collections::HashMap;
use std::time::Instant;
use tokio::time::{timeout, Duration};

use super::SandboxBackend;
use crate::sandbox::{SandboxRequest, SandboxResponse, SandboxFile};
use tracing::{info, warn, error, debug};

pub struct DockerBackend {
    docker: Docker,
}

impl DockerBackend {
    /// Execute a command in the container and capture output with detailed logging
    async fn execute_with_logging(&self, container_id: &str, command: &str, operation: &str) -> Result<(String, String, bool)> {
        info!("[DOCKER] Executing {} in container {}: {}", operation, container_id, command);
        
        let exec_options = CreateExecOptions {
            cmd: Some(vec!["sh", "-c", command]),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        };

        let exec = self.docker.create_exec(container_id, exec_options).await
            .context(format!("Failed to create exec for {}", operation))?;

        match self.docker.start_exec(&exec.id, None).await {
            Ok(StartExecResults::Attached { mut output, .. }) => {
                let mut stdout = String::new();
                let mut stderr = String::new();

                while let Some(chunk) = output.next().await {
                    match chunk {
                        Ok(bollard::container::LogOutput::StdOut { message }) => {
                            let output_str = String::from_utf8_lossy(&message);
                            stdout.push_str(&output_str);
                            debug!("[DOCKER] {} stdout: {}", operation, output_str.trim());
                        }
                        Ok(bollard::container::LogOutput::StdErr { message }) => {
                            let error_str = String::from_utf8_lossy(&message);
                            stderr.push_str(&error_str);
                            if !error_str.trim().is_empty() {
                                warn!("[DOCKER] {} stderr: {}", operation, error_str.trim());
                            }
                        }
                        Ok(_) => {}
                        Err(e) => {
                            error!("[DOCKER] {} stream error: {}", operation, e);
                            stderr.push_str(&format!("Stream error: {}", e));
                        }
                    }
                }

                let success = stderr.is_empty() || !stderr.contains("error") && !stderr.contains("Error") && !stderr.contains("ERROR");
                
                if success {
                    info!("[DOCKER] {} completed successfully", operation);
                    if !stdout.trim().is_empty() {
                        info!("[DOCKER] {} output: {}", operation, stdout.trim());
                    }
                } else {
                    error!("[DOCKER] {} failed with stderr: {}", operation, stderr.trim());
                }
                
                Ok((stdout, stderr, success))
            }
            Ok(StartExecResults::Detached) => {
                warn!("[DOCKER] {} execution detached unexpectedly", operation);
                Ok((String::new(), "Execution detached".to_string(), false))
            }
            Err(e) => {
                error!("[DOCKER] Failed to execute {}: {}", operation, e);
                Err(anyhow::anyhow!("Failed to execute {}: {}", operation, e))
            }
        }
    }

    pub fn new() -> Result<Self> {
        // Check for DOCKER_HOST environment variable, otherwise use local defaults
        let docker = if let Ok(docker_host) = std::env::var("DOCKER_HOST") {
            if docker_host.starts_with("tcp://") {
                let addr = docker_host.strip_prefix("tcp://").unwrap();
                Docker::connect_with_http(addr, 120, &ClientVersion { major_version: 1, minor_version: 41 })
                    .context("Failed to connect to Docker daemon with DOCKER_HOST")?
            } else {
                Docker::connect_with_local_defaults()
                    .context("Failed to connect to Docker daemon")?
            }
        } else {
            Docker::connect_with_local_defaults()
                .context("Failed to connect to Docker daemon")?
        };
        Ok(Self { docker })
    }

    fn find_available_port(&self) -> u16 {
        // Simple port allocation starting from 8080
        // In production, this should be more sophisticated
        use std::net::{TcpListener, SocketAddr};
        
        for port in 8080..9000 {
            let addr = SocketAddr::from(([127, 0, 0, 1], port));
            if TcpListener::bind(addr).is_ok() {
                return port;
            }
        }
        
        // Fallback to 8080 if nothing is available
        8080
    }

    async fn ensure_runtime_image(&self, runtime: &str) -> Result<String> {
        let image_name = match runtime {
            "node" | "nodejs" => "node:18-alpine",
            "bun" => "oven/bun:1-alpine",
            "typescript" | "ts" => "node:18-alpine",
            _ => anyhow::bail!("Unsupported runtime: {}", runtime),
        };

        let options = CreateImageOptions {
            from_image: image_name,
            ..Default::default()
        };

        let mut stream = self.docker.create_image(Some(options), None, None);
        while let Some(result) = stream.next().await {
            match result {
                Ok(_) => {}
                Err(e) => tracing::warn!("Image pull warning: {}", e),
            }
        }

        Ok(image_name.to_string())
    }

    async fn create_container(&self, request: &SandboxRequest, image: &str, host_port: Option<u16>) -> Result<(String, Option<u16>)> {
        // Auto-allocate port for dev servers if not provided
        let actual_host_port = if request.dev_server.unwrap_or(false) && matches!(request.mode, Some(crate::sandbox::SandboxMode::Persistent)) {
            host_port.or_else(|| Some(self.find_available_port()))
        } else {
            host_port
        };
        let mut env_vars = Vec::new();
        for (key, value) in &request.env_vars {
            env_vars.push(format!("{}={}", key, value));
        }

        let is_persistent = matches!(request.mode, Some(crate::sandbox::SandboxMode::Persistent));
        let has_dev_server = request.dev_server.unwrap_or(false);

        let config = Config {
            image: Some(image.to_string()),
            working_dir: Some("/sandbox".to_string()),
            env: Some(env_vars),
            cmd: if is_persistent {
                Some(vec!["tail".to_string(), "-f".to_string(), "/dev/null".to_string()])
            } else {
                None
            },
            host_config: Some(bollard::models::HostConfig {
                memory: Some((request.memory_limit_mb * 1024 * 1024) as i64),
                cpu_quota: Some(50000), // 50% CPU
                cpu_period: Some(100000),
                network_mode: if is_persistent && has_dev_server {
                    Some("bridge".to_string()) // Allow network for dev server
                } else {
                    Some("none".to_string()) // No network access
                },
                readonly_rootfs: Some(!is_persistent), // Allow writes for persistent mode
                port_bindings: if is_persistent && has_dev_server && actual_host_port.is_some() {
                    Some({
                        let mut port_bindings = HashMap::new();
                        port_bindings.insert(
                            "3000/tcp".to_string(),
                            Some(vec![bollard::models::PortBinding {
                                host_ip: Some("127.0.0.1".to_string()),
                                host_port: Some(actual_host_port.unwrap().to_string()),
                            }])
                        );
                        port_bindings
                    })
                } else {
                    None
                },
                tmpfs: Some({
                    let mut tmpfs = HashMap::new();
                    tmpfs.insert("/tmp".to_string(), "size=10m".to_string());
                    if is_persistent {
                        tmpfs.insert("/sandbox".to_string(), "size=500m".to_string());
                    } else {
                        tmpfs.insert("/sandbox".to_string(), "size=50m".to_string());
                    }
                    tmpfs
                }),
                ..Default::default()
            }),
            exposed_ports: if is_persistent && has_dev_server {
                Some({
                    let mut exposed_ports = HashMap::new();
                    exposed_ports.insert("3000/tcp".to_string(), HashMap::new());
                    exposed_ports
                })
            } else {
                None
            },
            ..Default::default()
        };

        let options = CreateContainerOptions {
            name: &request.id,
            platform: None,
        };

        let container = self
            .docker
            .create_container(Some(options), config)
            .await
            .context("Failed to create container")?;

        info!("[DOCKER] Container {} created with host port: {:?}", container.id, actual_host_port);
        Ok((container.id, actual_host_port))
    }


    /// Perform internal health check on the dev server
    async fn perform_health_check(&self, container_id: &str) -> Result<()> {
        info!("[DOCKER] Starting internal health check");
        
        // Check if any process is listening on port 3000
        let port_check_cmd = "netstat -tlnp 2>/dev/null | grep ':3000' || ss -tlnp 2>/dev/null | grep ':3000' || echo 'No process on port 3000'";
        let (port_output, _, _) = self.execute_with_logging(container_id, port_check_cmd, "port 3000 check").await?;
        
        if port_output.contains("No process on port 3000") {
            error!("[DOCKER] Health check FAILED: No process listening on port 3000");
            
            // Check what processes are running
            let ps_cmd = "ps aux | grep -E '(node|bun|npm)' | grep -v grep || echo 'No Node/Bun processes running'";
            let (ps_output, _, _) = self.execute_with_logging(container_id, ps_cmd, "process check").await?;
            warn!("[DOCKER] Running processes: {}", ps_output);
            
            return Err(anyhow::anyhow!("Health check failed: No service listening on port 3000"));
        } else {
            info!("[DOCKER] Health check: Process found on port 3000: {}", port_output.trim());
        }
        
        // Try to make an HTTP request to the service using wget (available in Alpine) or nc
        let http_check_cmd = "wget -q -O- --timeout=5 http://localhost:3000 2>/dev/null || nc -z localhost 3000 && echo 'PORT_ACCESSIBLE' || echo 'HTTP_CHECK_FAILED'";
        let (http_output, _, _) = self.execute_with_logging(container_id, http_check_cmd, "HTTP health check").await?;
        
        if http_output.contains("HTTP_CHECK_FAILED") {
            warn!("[DOCKER] Health check WARNING: HTTP request failed, but port is open");
            
            // Check if the service is still starting up using nc (netcat)
            let retry_cmd = "sleep 2 && nc -z localhost 3000 && echo 'PORT_ACCESSIBLE_RETRY' || echo 'HTTP_RETRY_FAILED'";
            let (retry_output, _, _) = self.execute_with_logging(container_id, retry_cmd, "HTTP retry check").await?;
            
            if retry_output.contains("HTTP_RETRY_FAILED") {
                error!("[DOCKER] Health check FAILED: Cannot connect to port 3000 after retry");
                return Err(anyhow::anyhow!("Health check failed: Service not responding on port 3000"));
            } else {
                info!("[DOCKER] Health check PASSED on retry: Port 3000 is accessible");
            }
        } else if http_output.contains("PORT_ACCESSIBLE") {
            info!("[DOCKER] Health check PASSED: Port 3000 is accessible");
        } else {
            info!("[DOCKER] Health check PASSED: HTTP response received: {}", http_output.trim());
        }
        
        info!("[DOCKER] Internal health check completed successfully");
        Ok(())
    }

    async fn execute_persistent_container(&self, container_id: &str, request: &SandboxRequest, start_time: Instant) -> Result<SandboxResponse> {
        // Create additional files if provided
        if let Some(files) = &request.files {
            // Create directories for nested files
            let mut directories = std::collections::HashSet::new();
            for file in files {
                if let Some(parent) = std::path::Path::new(&file.path).parent() {
                    if !parent.as_os_str().is_empty() && parent != std::path::Path::new(".") {
                        directories.insert(format!("/sandbox/{}", parent.display()));
                    }
                }
            }
            
            // Create directories
            for dir in directories {
                let mkdir_cmd = format!("mkdir -p {}", dir);
                let mkdir_exec_options = CreateExecOptions {
                    cmd: Some(vec!["sh", "-c", &mkdir_cmd]),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    ..Default::default()
                };
                let mkdir_exec = self.docker.create_exec(container_id, mkdir_exec_options).await?;
                if let Err(e) = self.docker.start_exec(&mkdir_exec.id, None).await {
                    tracing::error!("Failed to create directory {}: {}", dir, e);
                }
            }

            // Create files
            for file in files {
                let file_path = if file.path.starts_with('/') {
                    file.path.clone()
                } else {
                    format!("/sandbox/{}", file.path)
                };

                // Use proper escaping for file content
                let write_cmd = format!("cat > {} << 'EOF'\n{}\nEOF", file_path, file.content);

                let exec_options = CreateExecOptions {
                    cmd: Some(vec!["sh", "-c", &write_cmd]),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    ..Default::default()
                };

                let exec = self.docker.create_exec(container_id, exec_options).await?;
                if let Err(e) = self.docker.start_exec(&exec.id, None).await {
                    tracing::error!("Failed to create file {}: {}", file.path, e);
                }

                // Make executable if specified
                if file.is_executable.unwrap_or(false) {
                    let chmod_cmd = format!("chmod +x {}", file_path);

                    let chmod_exec_options = CreateExecOptions {
                        cmd: Some(vec!["sh", "-c", &chmod_cmd]),
                        attach_stdout: Some(true),
                        attach_stderr: Some(true),
                        ..Default::default()
                    };

                    let chmod_exec = self.docker.create_exec(container_id, chmod_exec_options).await?;
                    if let Err(e) = self.docker.start_exec(&chmod_exec.id, None).await {
                        tracing::error!("Failed to chmod file {}: {}", file.path, e);
                    }
                }
            }
        }

        // Write main code to file if not provided in files
        if request.files.is_none() || !request.files.as_ref().unwrap().iter().any(|f| f.path.contains("index") || f.path.contains("main")) {
            let code_file = match request.runtime.as_str() {
                "bun" => "/sandbox/index.js",
                "node" | "nodejs" => "/sandbox/index.js", 
                "typescript" | "ts" => "/sandbox/index.ts",
                _ => "/sandbox/index.js",
            };
            
            let write_code_cmd = format!("cat > {} << 'EOF'\n{}\nEOF", code_file, request.code);

            let exec_options = CreateExecOptions {
                cmd: Some(vec!["sh", "-c", &write_code_cmd]),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                ..Default::default()
            };

            let exec = self.docker.create_exec(container_id, exec_options).await?;
            if let Err(e) = self.docker.start_exec(&exec.id, None).await {
                tracing::error!("Failed to write main code file: {}", e);
            }
        }

        // Install dependencies if requested
        if request.install_deps.unwrap_or(false) || request.dev_server.unwrap_or(false) {
            info!("[DOCKER] Installing dependencies for {} runtime", request.runtime);
            
            // Check if package.json exists first
            let check_package_cmd = "test -f /sandbox/package.json && echo 'package.json found' || echo 'package.json not found'";
            let (check_output, _, _) = self.execute_with_logging(container_id, check_package_cmd, "package.json check").await?;
            info!("[DOCKER] Package check result: {}", check_output.trim());
            
            // Auto-create package.json if none exists and we're using Bun or Node
            if check_output.contains("package.json not found") {
                info!("[DOCKER] Auto-creating package.json for {} runtime", request.runtime);
                
                let package_json_content = match request.runtime.as_str() {
                    "bun" => {
                        r#"{
  "name": "faas-bun-app",
  "version": "1.0.0",
  "type": "module",
  "scripts": {
    "dev": "bun run index.js",
    "start": "bun run index.js"
  },
  "dependencies": {},
  "devDependencies": {}
}"#
                    }
                    "node" | "nodejs" => {
                        r#"{
  "name": "faas-node-app",
  "version": "1.0.0",
  "main": "index.js",
  "scripts": {
    "dev": "node index.js",
    "start": "node index.js"
  },
  "dependencies": {},
  "devDependencies": {}
}"#
                    }
                    _ => {
                        r#"{
  "name": "faas-app",
  "version": "1.0.0",
  "main": "index.js",
  "scripts": {
    "dev": "node index.js",
    "start": "node index.js"
  },
  "dependencies": {},
  "devDependencies": {}
}"#
                    }
                };
                
                let create_package_cmd = format!("cat > /sandbox/package.json << 'EOF'\n{}\nEOF", package_json_content);
                match self.execute_with_logging(container_id, &create_package_cmd, "package.json creation").await {
                    Ok((_, _, success)) => {
                        if success {
                            info!("[DOCKER] package.json created successfully");
                        } else {
                            error!("[DOCKER] Failed to create package.json");
                            return Err(anyhow::anyhow!("Failed to create package.json"));
                        }
                    }
                    Err(e) => {
                        error!("[DOCKER] Error creating package.json: {}", e);
                        return Err(e);
                    }
                }
            }
            
            // Now proceed with dependency installation
            let install_cmd = match request.runtime.as_str() {
                "bun" => {
                    info!("[DOCKER] Using Bun package manager for dependency installation");
                    "cd /sandbox && bun install --verbose"
                }
                "node" | "nodejs" => {
                    info!("[DOCKER] Using npm package manager for dependency installation");
                    "cd /sandbox && npm install --verbose"
                }
                _ => {
                    warn!("[DOCKER] Unknown runtime {}, defaulting to npm", request.runtime);
                    "cd /sandbox && npm install --verbose"
                }
            };

            match self.execute_with_logging(container_id, install_cmd, "dependency installation").await {
                Ok((stdout, stderr, success)) => {
                    if success {
                        info!("[DOCKER] Dependencies installed successfully");
                        
                        // Log dependency count if available
                        let count_cmd = "cd /sandbox && find node_modules -maxdepth 1 -type d | wc -l || echo 'node_modules count failed'";
                        if let Ok((count_output, _, _)) = self.execute_with_logging(container_id, count_cmd, "dependency count").await {
                            info!("[DOCKER] Installed dependencies count: {}", count_output.trim());
                        }
                    } else {
                        error!("[DOCKER] Dependency installation failed!");
                        error!("[DOCKER] Install stdout: {}", stdout);
                        error!("[DOCKER] Install stderr: {}", stderr);
                        return Err(anyhow::anyhow!("Dependency installation failed: {}", stderr));
                    }
                }
                Err(e) => {
                    error!("[DOCKER] Failed to execute dependency installation: {}", e);
                    return Err(e);
                }
            }
        }

        // Start development server if requested
        if request.dev_server.unwrap_or(false) {
            info!("[DOCKER] Starting development server");
            
            let dev_cmd = if let Some(entry_point) = &request.entry_point {
                info!("[DOCKER] Using custom entry point: {}", entry_point);
                format!("cd /sandbox && {}", entry_point)
            } else {
                let default_cmd = match request.runtime.as_str() {
                    "bun" => "cd /sandbox && bun dev".to_string(),
                    "node" | "nodejs" => "cd /sandbox && npm run dev".to_string(),
                    _ => "cd /sandbox && bun dev".to_string(),
                };
                info!("[DOCKER] Using default dev command for {}: {}", request.runtime, default_cmd);
                default_cmd
            };

            // Check if the command exists in package.json (for npm/bun)
            if !dev_cmd.contains("node ") && !dev_cmd.contains("bun run /") {
                let check_script_cmd = "cd /sandbox && cat package.json | grep -o '\"dev\"' || echo 'no dev script'";
                let (script_check, _, _) = self.execute_with_logging(container_id, check_script_cmd, "dev script check").await?;
                info!("[DOCKER] Dev script availability: {}", script_check.trim());
            }

            // Start dev server in background and capture initial output
            info!("[DOCKER] Starting dev server with command: {}", dev_cmd);
            let dev_cmd_bg = format!("{} > /sandbox/dev-server.log 2>&1 &", dev_cmd);
            
            match self.execute_with_logging(container_id, &dev_cmd_bg, "dev server startup").await {
                Ok((stdout, stderr, success)) => {
                    if !success {
                        error!("[DOCKER] Dev server startup command failed!");
                        error!("[DOCKER] Startup stdout: {}", stdout);
                        error!("[DOCKER] Startup stderr: {}", stderr);
                    } else {
                        info!("[DOCKER] Dev server startup command executed");
                    }
                }
                Err(e) => {
                    error!("[DOCKER] Failed to start dev server: {}", e);
                }
            }

            // Wait for the server to start
            info!("[DOCKER] Waiting for dev server to initialize...");
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            
            // Check dev server logs
            let log_cmd = "cd /sandbox && tail -20 dev-server.log 2>/dev/null || echo 'No dev server logs found'";
            match self.execute_with_logging(container_id, log_cmd, "dev server logs check").await {
                Ok((log_output, _, _)) => {
                    if !log_output.trim().is_empty() && log_output != "No dev server logs found" {
                        info!("[DOCKER] Dev server logs:\n{}", log_output);
                    } else {
                        warn!("[DOCKER] No dev server logs found");
                    }
                }
                Err(e) => {
                    warn!("[DOCKER] Failed to read dev server logs: {}", e);
                }
            }
            
            // Perform health check
            self.perform_health_check(container_id).await?;
        }

        // Container is already running with tail -f /dev/null as the main process

        let execution_time = start_time.elapsed().as_millis() as u64;

        let final_status = if request.dev_server.unwrap_or(false) {
            "Persistent container with dev server setup completed"
        } else {
            "Persistent container setup completed"
        };
        
        info!("[DOCKER] {}", final_status);
        
        Ok(SandboxResponse {
            success: true,
            stdout: final_status.to_string(),
            stderr: String::new(),
            exit_code: Some(0),
            execution_time_ms: execution_time,
            is_running: Some(true),
            dev_server_url: Some("http://localhost:3000".to_string()),
        })
    }

    async fn execute_in_container(&self, container_id: &str, request: &SandboxRequest) -> Result<SandboxResponse> {
        let start_time = Instant::now();
        
        // For persistent mode, we need to handle things differently
        let is_persistent = matches!(request.mode, Some(crate::sandbox::SandboxMode::Persistent));
        
        if is_persistent {
            return self.execute_persistent_container(container_id, request, start_time).await;
        }

        // Create additional files if provided
        if let Some(files) = &request.files {
            for file in files {
                let file_cmd = if file.path.starts_with('/') {
                    format!("echo '{}' > {}", file.content.replace('\'', "'\"'\"'"), file.path)
                } else {
                    format!("echo '{}' > /sandbox/{}", file.content.replace('\'', "'\"'\"'"), file.path)
                };

                let exec_options = CreateExecOptions {
                    cmd: Some(vec!["sh", "-c", &file_cmd]),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    ..Default::default()
                };

                let exec = self.docker.create_exec(container_id, exec_options).await?;
                self.docker.start_exec(&exec.id, None).await?;

                // Make executable if specified
                if file.is_executable.unwrap_or(false) {
                    let chmod_cmd = if file.path.starts_with('/') {
                        format!("chmod +x {}", file.path)
                    } else {
                        format!("chmod +x /sandbox/{}", file.path)
                    };

                    let chmod_exec_options = CreateExecOptions {
                        cmd: Some(vec!["sh", "-c", &chmod_cmd]),
                        attach_stdout: Some(true),
                        attach_stderr: Some(true),
                        ..Default::default()
                    };

                    let chmod_exec = self.docker.create_exec(container_id, chmod_exec_options).await?;
                    self.docker.start_exec(&chmod_exec.id, None).await?;
                }
            }
        }

        // Write code to container
        let code_write_cmd = match request.runtime.as_str() {
            "node" | "nodejs" => {
                format!("echo '{}' > /sandbox/index.js", request.code.replace('\'', "'\"'\"'"))
            }
            "bun" => {
                format!("echo '{}' > /sandbox/index.js", request.code.replace('\'', "'\"'\"'"))
            }
            "typescript" | "ts" => {
                format!("echo '{}' > /sandbox/index.ts", request.code.replace('\'', "'\"'\"'"))
            }
            _ => anyhow::bail!("Unsupported runtime: {}", request.runtime),
        };

        let exec_options = CreateExecOptions {
            cmd: Some(vec!["sh", "-c", &code_write_cmd]),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        };

        let exec = self
            .docker
            .create_exec(container_id, exec_options)
            .await
            .context("Failed to create exec for writing code")?;

        self.docker
            .start_exec(&exec.id, None)
            .await
            .context("Failed to write code to container")?;

        // Execute code
        let run_cmd = match request.runtime.as_str() {
            "node" | "nodejs" => "node /sandbox/index.js",
            "bun" => "bun run /sandbox/index.js",
            "typescript" | "ts" => "npx ts-node /sandbox/index.ts",
            _ => anyhow::bail!("Unsupported runtime: {}", request.runtime),
        };

        let exec_options = CreateExecOptions {
            cmd: Some(vec!["sh", "-c", run_cmd]),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        };

        let exec = self
            .docker
            .create_exec(container_id, exec_options)
            .await
            .context("Failed to create exec for running code")?;

        let timeout_duration = Duration::from_millis(request.timeout_ms);
        let exec_result = timeout(timeout_duration, self.docker.start_exec(&exec.id, None)).await;

        let execution_time = start_time.elapsed().as_millis() as u64;

        match exec_result {
            Ok(Ok(StartExecResults::Attached { mut output, .. })) => {
                let mut stdout = String::new();
                let mut stderr = String::new();

                while let Some(chunk) = output.next().await {
                    match chunk {
                        Ok(bollard::container::LogOutput::StdOut { message }) => {
                            stdout.push_str(&String::from_utf8_lossy(&message));
                        }
                        Ok(bollard::container::LogOutput::StdErr { message }) => {
                            stderr.push_str(&String::from_utf8_lossy(&message));
                        }
                        Ok(_) => {}
                        Err(e) => {
                            stderr.push_str(&format!("Stream error: {}", e));
                        }
                    }
                }

                let success = stderr.is_empty();
                Ok(SandboxResponse {
                    success,
                    stdout,
                    stderr,
                    exit_code: Some(if success { 0 } else { 1 }),
                    execution_time_ms: execution_time,
                    is_running: Some(false),
                    dev_server_url: None,
                })
            }
            Ok(Ok(StartExecResults::Detached)) => {
                Ok(SandboxResponse {
                    success: false,
                    stdout: String::new(),
                    stderr: "Execution detached unexpectedly".to_string(),
                    exit_code: Some(1),
                    execution_time_ms: execution_time,
                    is_running: Some(false),
                    dev_server_url: None,
                })
            }
            Ok(Err(e)) => {
                Ok(SandboxResponse {
                    success: false,
                    stdout: String::new(),
                    stderr: format!("Execution error: {}", e),
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
}

#[async_trait]
impl SandboxBackend for DockerBackend {
    async fn create_sandbox(&self, request: &SandboxRequest) -> Result<()> {
        let image = self.ensure_runtime_image(&request.runtime).await?;
        let (container_id, allocated_port) = self.create_container(request, &image, None).await?;
        
        if let Some(port) = allocated_port {
            info!("[DOCKER] Sandbox {} allocated host port {}", request.id, port);
            // TODO: Store port mapping for proxy access
        }
        
        self.docker
            .start_container(&container_id, None::<StartContainerOptions<String>>)
            .await
            .context("Failed to start container")?;

        Ok(())
    }

    async fn execute_sandbox(&self, request: &SandboxRequest) -> Result<SandboxResponse> {
        let response = self.execute_in_container(&request.id, request).await?;
        Ok(response)
    }

    async fn cleanup_sandbox(&self, sandbox_id: &str) -> Result<()> {
        let options = RemoveContainerOptions {
            force: true,
            ..Default::default()
        };

        self.docker
            .remove_container(sandbox_id, Some(options))
            .await
            .context("Failed to remove container")?;

        Ok(())
    }

    async fn is_available(&self) -> bool {
        self.docker.ping().await.is_ok()
    }

    
    async fn update_files(&self, sandbox_id: &str, files: &[SandboxFile]) -> Result<()> {
        for file in files {
            // Create directories if needed
            if let Some(parent) = std::path::Path::new(&file.path).parent() {
                if !parent.as_os_str().is_empty() && parent != std::path::Path::new(".") {
                    let mkdir_cmd = format!("mkdir -p /sandbox/{}", parent.display());
                    let mkdir_exec_options = CreateExecOptions {
                        cmd: Some(vec!["sh", "-c", &mkdir_cmd]),
                        attach_stdout: Some(true),
                        attach_stderr: Some(true),
                        ..Default::default()
                    };
                    let mkdir_exec = self.docker.create_exec(sandbox_id, mkdir_exec_options).await?;
                    if let Err(e) = self.docker.start_exec(&mkdir_exec.id, None).await {
                        warn!("Failed to create directory for {}: {}", file.path, e);
                    }
                }
            }

            // Write file content
            let file_path = format!("/sandbox/{}", file.path);
            let write_cmd = format!("cat > {} << 'EOF'\n{}\nEOF", file_path, file.content);

            let exec_options = CreateExecOptions {
                cmd: Some(vec!["sh", "-c", &write_cmd]),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                ..Default::default()
            };

            let exec = self.docker.create_exec(sandbox_id, exec_options).await?;
            self.docker.start_exec(&exec.id, None).await
                .map_err(|e| anyhow::anyhow!("Failed to update file {}: {}", file.path, e))?;

            // Make executable if specified
            if file.is_executable.unwrap_or(false) {
                let chmod_cmd = format!("chmod +x {}", file_path);
                let chmod_exec_options = CreateExecOptions {
                    cmd: Some(vec!["sh", "-c", &chmod_cmd]),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    ..Default::default()
                };
                let chmod_exec = self.docker.create_exec(sandbox_id, chmod_exec_options).await?;
                if let Err(e) = self.docker.start_exec(&chmod_exec.id, None).await {
                    warn!("Failed to chmod file {}: {}", file.path, e);
                }
            }

            info!("Updated file: /sandbox/{}", file.path);
        }
        Ok(())
    }
    
    async fn restart_process(&self, sandbox_id: &str, command: &str) -> Result<()> {
        // Kill existing processes that match the command pattern
        let kill_cmd = match command {
            cmd if cmd.contains("bun") => "pkill -f 'bun.*dev' || true",
            cmd if cmd.contains("npm") => "pkill -f 'npm.*run' || true", 
            cmd if cmd.contains("node") => "pkill -f 'node.*' || true",
            _ => "pkill -f 'dev' || true",
        };
        
        let kill_exec_options = CreateExecOptions {
            cmd: Some(vec!["sh", "-c", kill_cmd]),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        };
        let kill_exec = self.docker.create_exec(sandbox_id, kill_exec_options).await?;
        self.docker.start_exec(&kill_exec.id, None).await?;

        // Wait a moment for processes to stop
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Start new process in background
        let bg_cmd = format!("cd /sandbox && nohup {} > /sandbox/dev-server.log 2>&1 &", command);
        let dev_exec_options = CreateExecOptions {
            cmd: Some(vec!["sh", "-c", &bg_cmd]),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        };

        let dev_exec = self.docker.create_exec(sandbox_id, dev_exec_options).await?;
        self.docker.start_exec(&dev_exec.id, None).await
            .map_err(|e| anyhow::anyhow!("Failed to restart process: {}", e))?;

        info!("Restarted process '{}' for sandbox {}", command, sandbox_id);
        Ok(())
    }
}