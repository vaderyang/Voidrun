#[cfg(feature = "docker")]
use anyhow::{Context, Result};
use async_trait::async_trait;
use bollard::{
    container::{Config, CreateContainerOptions, RemoveContainerOptions, StartContainerOptions},
    exec::{CreateExecOptions, StartExecResults},
    image::CreateImageOptions,
    Docker,
};
use futures_util::StreamExt;
use std::collections::HashMap;
use std::time::Instant;
use tokio::time::{timeout, Duration};

use super::{SandboxBackend, SandboxBackendType};
use crate::sandbox::{SandboxRequest, SandboxResponse, SandboxFile};
use tracing::{info, warn};

pub struct DockerBackend {
    docker: Docker,
}

impl DockerBackend {
    pub fn new() -> Result<Self> {
        let docker = Docker::connect_with_local_defaults()
            .context("Failed to connect to Docker daemon")?;
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

    async fn create_container(&self, request: &SandboxRequest, image: &str, host_port: Option<u16>) -> Result<String> {
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

        Ok(container.id)
    }

    async fn setup_react_dev_environment(&self, container_id: &str, request: &SandboxRequest) -> Result<()> {
        // Create package.json for Vite + React project
        let package_json = r#"{
  "name": "sandbox-react-app",
  "private": true,
  "version": "0.0.0",
  "type": "module",
  "scripts": {
    "dev": "vite --host 0.0.0.0 --port 3000",
    "build": "vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0"
  },
  "devDependencies": {
    "@types/react": "^18.2.15",
    "@types/react-dom": "^18.2.7",
    "@vitejs/plugin-react": "^4.0.3",
    "autoprefixer": "^10.4.14",
    "postcss": "^8.4.27",
    "tailwindcss": "^3.3.3",
    "vite": "^4.4.5"
  }
}"#;

        // Create vite.config.js
        let vite_config = r#"import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  server: {
    host: '0.0.0.0',
    port: 3000
  }
})
"#;

        // Create tailwind.config.js
        let tailwind_config = r#"/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {},
  },
  plugins: [],
}
"#;

        // Create postcss.config.js
        let postcss_config = r#"export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
}
"#;

        // Create index.html
        let index_html = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <link rel="icon" type="image/svg+xml" href="/vite.svg" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Sandbox React App</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
"#;

        // Create src/main.tsx
        let main_tsx = r#"import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App.tsx'
import './index.css'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
"#;

        // Handle special case where code field contains JSON with reactCode and cssCode
        let (react_code, css_code) = if request.code.trim().starts_with('{') && request.code.contains("reactCode") {
            // Parse JSON to extract reactCode and cssCode
            match serde_json::from_str::<serde_json::Value>(&request.code) {
                Ok(json) => {
                    let react_code = json.get("reactCode")
                        .and_then(|v| v.as_str())
                        .unwrap_or("export default function App() { return <div>Hello World</div>; }")
                        .to_string();
                    let css_code = json.get("cssCode")
                        .and_then(|v| v.as_str())
                        .unwrap_or("@tailwind base;\n@tailwind components;\n@tailwind utilities;")
                        .to_string();
                    (react_code, css_code)
                },
                Err(_) => {
                    // Fallback to treating code as React component
                    (request.code.clone(), "@tailwind base;\n@tailwind components;\n@tailwind utilities;".to_string())
                }
            }
        } else {
            // Regular React code
            (request.code.clone(), "@tailwind base;\n@tailwind components;\n@tailwind utilities;".to_string())
        };

        // Create src/index.css with the provided CSS or default
        let index_css = if let Some(files) = &request.files {
            if let Some(css_file) = files.iter().find(|f| f.path.contains("css") || f.path == "cssCode") {
                &css_file.content
            } else {
                &css_code
            }
        } else {
            &css_code
        };

        // Create src/App.tsx with the provided React code or default
        let app_tsx = if let Some(files) = &request.files {
            if let Some(react_file) = files.iter().find(|f| f.path.contains("tsx") || f.path.contains("jsx") || f.path == "reactCode") {
                &react_file.content
            } else {
                &react_code
            }
        } else {
            &react_code
        };

        // Write all files to the container
        let files_to_create = vec![
            ("/sandbox/package.json", package_json),
            ("/sandbox/vite.config.js", vite_config),
            ("/sandbox/tailwind.config.js", tailwind_config),
            ("/sandbox/postcss.config.js", postcss_config),
            ("/sandbox/index.html", index_html),
            ("/sandbox/src/main.tsx", main_tsx),
            ("/sandbox/src/index.css", index_css),
            ("/sandbox/src/App.tsx", app_tsx),
        ];

        // Create src directory first
        let mkdir_cmd = "mkdir -p /sandbox/src";
        let mkdir_exec_options = CreateExecOptions {
            cmd: Some(vec!["sh", "-c", mkdir_cmd]),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        };
        let mkdir_exec = self.docker.create_exec(container_id, mkdir_exec_options).await?;
        self.docker.start_exec(&mkdir_exec.id, None).await?;

        // Write all files
        for (file_path, content) in files_to_create {
            let write_cmd = format!("cat > {} << 'EOF'\n{}\nEOF", file_path, content);
            let exec_options = CreateExecOptions {
                cmd: Some(vec!["sh", "-c", &write_cmd]),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                ..Default::default()
            };
            let exec = self.docker.create_exec(container_id, exec_options).await?;
            self.docker.start_exec(&exec.id, None).await?;
        }

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
            let install_cmd = match request.runtime.as_str() {
                "bun" => "cd /sandbox && bun install",
                "node" | "nodejs" => "cd /sandbox && npm install",
                _ => "cd /sandbox && npm install",
            };

            let install_exec_options = CreateExecOptions {
                cmd: Some(vec!["sh", "-c", install_cmd]),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                ..Default::default()
            };

            let install_exec = self.docker.create_exec(container_id, install_exec_options).await?;
            if let Err(e) = self.docker.start_exec(&install_exec.id, None).await {
                tracing::error!("Failed to install dependencies: {}", e);
            }
        }

        // Start development server if requested
        if request.dev_server.unwrap_or(false) {
            let dev_cmd = if let Some(entry_point) = &request.entry_point {
                format!("cd /sandbox && {}", entry_point)
            } else {
                match request.runtime.as_str() {
                    "bun" => "cd /sandbox && bun dev".to_string(),
                    "node" | "nodejs" => "cd /sandbox && npm run dev".to_string(),
                    _ => "cd /sandbox && bun dev".to_string(),
                }
            };

            // Start dev server in background - use nohup to keep it running
            let dev_cmd_bg = format!("nohup {} > /sandbox/dev-server.log 2>&1 &", dev_cmd);
            let dev_exec_options = CreateExecOptions {
                cmd: Some(vec!["sh", "-c", &dev_cmd_bg]),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                ..Default::default()
            };

            let dev_exec = self.docker.create_exec(container_id, dev_exec_options).await?;
            if let Err(e) = self.docker.start_exec(&dev_exec.id, None).await {
                tracing::error!("Failed to start dev server: {}", e);
            }

            // Wait a moment for the server to start
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        }

        // Container is already running with tail -f /dev/null as the main process

        let execution_time = start_time.elapsed().as_millis() as u64;

        Ok(SandboxResponse {
            success: true,
            stdout: "Persistent container setup completed".to_string(),
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
        let container_id = self.create_container(request, &image, None).await?;
        
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

    fn backend_type(&self) -> SandboxBackendType {
        SandboxBackendType::Docker
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