use sandbox_service::sandbox::backend::{SandboxBackend, SandboxBackendType, create_backend};
use sandbox_service::sandbox::{SandboxRequest, SandboxResponse};
use std::collections::HashMap;
use tempfile::TempDir;
use uuid::Uuid;

#[cfg(test)]
mod nsjail_tests {
    use super::*;

    fn create_test_request(runtime: &str, code: &str) -> SandboxRequest {
        SandboxRequest {
            id: Uuid::new_v4().to_string(),
            runtime: runtime.to_string(),
            code: code.to_string(),
            entry_point: None,
            timeout_ms: 5000,
            memory_limit_mb: 128,
            env_vars: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_nsjail_availability() {
        let backend = create_backend(SandboxBackendType::Nsjail);
        
        match backend {
            Ok(backend) => {
                let is_available = backend.is_available().await;
                // If nsjail is not installed, this might fail
                // We'll just check that the method doesn't panic
                println!("nsjail availability: {}", is_available);
            }
            Err(e) => {
                println!("nsjail backend creation failed: {}", e);
                // This is expected if nsjail is not installed
            }
        }
    }

    #[tokio::test]
    async fn test_nsjail_node_execution() {
        let backend = create_backend(SandboxBackendType::Nsjail);
        
        if let Ok(backend) = backend {
            if backend.is_available().await {
                let request = create_test_request("node", "console.log('Hello from nsjail!');");
                
                let create_result = backend.create_sandbox(&request).await;
                assert!(create_result.is_ok());
                
                let execute_result = backend.execute_sandbox(&request).await;
                assert!(execute_result.is_ok());
                
                let response = execute_result.unwrap();
                assert!(response.success);
                assert!(response.stdout.contains("Hello from nsjail!"));
                assert!(response.stderr.is_empty());
                assert_eq!(response.exit_code, Some(0));
                
                let cleanup_result = backend.cleanup_sandbox(&request.id).await;
                assert!(cleanup_result.is_ok());
            } else {
                println!("nsjail not available, skipping test");
            }
        } else {
            println!("nsjail backend not available, skipping test");
        }
    }

    #[tokio::test]
    async fn test_nsjail_error_handling() {
        let backend = create_backend(SandboxBackendType::Nsjail);
        
        if let Ok(backend) = backend {
            if backend.is_available().await {
                let request = create_test_request("node", "throw new Error('Test error');");
                
                let create_result = backend.create_sandbox(&request).await;
                assert!(create_result.is_ok());
                
                let execute_result = backend.execute_sandbox(&request).await;
                assert!(execute_result.is_ok());
                
                let response = execute_result.unwrap();
                assert!(!response.success);
                assert!(response.stderr.contains("Error: Test error"));
                assert_eq!(response.exit_code, Some(1));
                
                let cleanup_result = backend.cleanup_sandbox(&request.id).await;
                assert!(cleanup_result.is_ok());
            } else {
                println!("nsjail not available, skipping test");
            }
        } else {
            println!("nsjail backend not available, skipping test");
        }
    }

    #[tokio::test]
    async fn test_nsjail_timeout() {
        let backend = create_backend(SandboxBackendType::Nsjail);
        
        if let Ok(backend) = backend {
            if backend.is_available().await {
                let mut request = create_test_request("node", "while(true) {}");
                request.timeout_ms = 1000; // 1 second timeout
                
                let create_result = backend.create_sandbox(&request).await;
                assert!(create_result.is_ok());
                
                let execute_result = backend.execute_sandbox(&request).await;
                assert!(execute_result.is_ok());
                
                let response = execute_result.unwrap();
                assert!(!response.success);
                assert!(response.stderr.contains("timed out"));
                assert_eq!(response.exit_code, Some(124));
                
                let cleanup_result = backend.cleanup_sandbox(&request.id).await;
                assert!(cleanup_result.is_ok());
            } else {
                println!("nsjail not available, skipping test");
            }
        } else {
            println!("nsjail backend not available, skipping test");
        }
    }
}

#[cfg(test)]
mod docker_tests {
    use super::*;

    fn create_test_request(runtime: &str, code: &str) -> SandboxRequest {
        SandboxRequest {
            id: Uuid::new_v4().to_string(),
            runtime: runtime.to_string(),
            code: code.to_string(),
            entry_point: None,
            timeout_ms: 5000,
            memory_limit_mb: 128,
            env_vars: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_docker_availability() {
        let backend = create_backend(SandboxBackendType::Docker);
        
        match backend {
            Ok(backend) => {
                let is_available = backend.is_available().await;
                println!("Docker availability: {}", is_available);
                
                if !is_available {
                    println!("Docker daemon is not running or not accessible");
                }
            }
            Err(e) => {
                println!("Docker backend creation failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_docker_node_execution() {
        let backend = create_backend(SandboxBackendType::Docker);
        
        if let Ok(backend) = backend {
            if backend.is_available().await {
                let request = create_test_request("node", "console.log('Hello from Docker!');");
                
                let create_result = backend.create_sandbox(&request).await;
                assert!(create_result.is_ok());
                
                let execute_result = backend.execute_sandbox(&request).await;
                assert!(execute_result.is_ok());
                
                let response = execute_result.unwrap();
                assert!(response.success);
                assert!(response.stdout.contains("Hello from Docker!"));
                assert!(response.stderr.is_empty());
                assert_eq!(response.exit_code, Some(0));
                
                let cleanup_result = backend.cleanup_sandbox(&request.id).await;
                assert!(cleanup_result.is_ok());
            } else {
                println!("Docker not available, skipping test");
            }
        } else {
            println!("Docker backend not available, skipping test");
        }
    }

    #[tokio::test]
    async fn test_docker_typescript_execution() {
        let backend = create_backend(SandboxBackendType::Docker);
        
        if let Ok(backend) = backend {
            if backend.is_available().await {
                let request = create_test_request(
                    "typescript",
                    "interface User { name: string; } const user: User = { name: 'Docker' }; console.log(`Hello, ${user.name}!`);"
                );
                
                let create_result = backend.create_sandbox(&request).await;
                assert!(create_result.is_ok());
                
                let execute_result = backend.execute_sandbox(&request).await;
                assert!(execute_result.is_ok());
                
                let response = execute_result.unwrap();
                assert!(response.success);
                assert!(response.stdout.contains("Hello, Docker!"));
                
                let cleanup_result = backend.cleanup_sandbox(&request.id).await;
                assert!(cleanup_result.is_ok());
            } else {
                println!("Docker not available, skipping test");
            }
        } else {
            println!("Docker backend not available, skipping test");
        }
    }

    #[tokio::test]
    async fn test_docker_error_handling() {
        let backend = create_backend(SandboxBackendType::Docker);
        
        if let Ok(backend) = backend {
            if backend.is_available().await {
                let request = create_test_request("node", "throw new Error('Docker test error');");
                
                let create_result = backend.create_sandbox(&request).await;
                assert!(create_result.is_ok());
                
                let execute_result = backend.execute_sandbox(&request).await;
                assert!(execute_result.is_ok());
                
                let response = execute_result.unwrap();
                assert!(!response.success);
                assert!(response.stderr.contains("Error: Docker test error"));
                assert_eq!(response.exit_code, Some(1));
                
                let cleanup_result = backend.cleanup_sandbox(&request.id).await;
                assert!(cleanup_result.is_ok());
            } else {
                println!("Docker not available, skipping test");
            }
        } else {
            println!("Docker backend not available, skipping test");
        }
    }

    #[tokio::test]
    async fn test_docker_environment_variables() {
        let backend = create_backend(SandboxBackendType::Docker);
        
        if let Ok(backend) = backend {
            if backend.is_available().await {
                let mut request = create_test_request(
                    "node",
                    "console.log('TEST_VAR:', process.env.TEST_VAR); console.log('NODE_ENV:', process.env.NODE_ENV);"
                );
                request.env_vars.insert("TEST_VAR".to_string(), "docker_test".to_string());
                request.env_vars.insert("NODE_ENV".to_string(), "sandbox".to_string());
                
                let create_result = backend.create_sandbox(&request).await;
                assert!(create_result.is_ok());
                
                let execute_result = backend.execute_sandbox(&request).await;
                assert!(execute_result.is_ok());
                
                let response = execute_result.unwrap();
                assert!(response.success);
                assert!(response.stdout.contains("TEST_VAR: docker_test"));
                assert!(response.stdout.contains("NODE_ENV: sandbox"));
                
                let cleanup_result = backend.cleanup_sandbox(&request.id).await;
                assert!(cleanup_result.is_ok());
            } else {
                println!("Docker not available, skipping test");
            }
        } else {
            println!("Docker backend not available, skipping test");
        }
    }
}

#[cfg(test)]
mod backend_comparison_tests {
    use super::*;

    async fn test_backend_with_code(backend_type: SandboxBackendType, code: &str) -> Option<SandboxResponse> {
        let backend = create_backend(backend_type);
        
        if let Ok(backend) = backend {
            if backend.is_available().await {
                let request = SandboxRequest {
                    id: Uuid::new_v4().to_string(),
                    runtime: "node".to_string(),
                    code: code.to_string(),
                    entry_point: None,
                    timeout_ms: 5000,
                    memory_limit_mb: 128,
                    env_vars: HashMap::new(),
                };
                
                if backend.create_sandbox(&request).await.is_ok() {
                    if let Ok(response) = backend.execute_sandbox(&request).await {
                        let _ = backend.cleanup_sandbox(&request.id).await;
                        return Some(response);
                    }
                }
            }
        }
        None
    }

    #[tokio::test]
    async fn test_backend_consistency() {
        let test_code = "console.log('Hello, World!'); console.log('Line 2'); console.log(42);";
        
        let nsjail_result = test_backend_with_code(SandboxBackendType::Nsjail, test_code).await;
        let docker_result = test_backend_with_code(SandboxBackendType::Docker, test_code).await;
        
        match (nsjail_result, docker_result) {
            (Some(nsjail), Some(docker)) => {
                assert_eq!(nsjail.success, docker.success);
                assert_eq!(nsjail.stdout, docker.stdout);
                assert_eq!(nsjail.exit_code, docker.exit_code);
                println!("Both backends produced consistent results");
            }
            (Some(nsjail), None) => {
                assert!(nsjail.success);
                println!("Only nsjail backend available");
            }
            (None, Some(docker)) => {
                assert!(docker.success);
                println!("Only Docker backend available");
            }
            (None, None) => {
                println!("No backends available for testing");
            }
        }
    }

    #[tokio::test]
    async fn test_error_consistency() {
        let test_code = "throw new Error('Test error message');";
        
        let nsjail_result = test_backend_with_code(SandboxBackendType::Nsjail, test_code).await;
        let docker_result = test_backend_with_code(SandboxBackendType::Docker, test_code).await;
        
        match (nsjail_result, docker_result) {
            (Some(nsjail), Some(docker)) => {
                assert_eq!(nsjail.success, docker.success);
                assert!(!nsjail.success);
                assert!(!docker.success);
                assert!(nsjail.stderr.contains("Test error message"));
                assert!(docker.stderr.contains("Test error message"));
                assert_eq!(nsjail.exit_code, docker.exit_code);
                println!("Both backends handle errors consistently");
            }
            (Some(nsjail), None) => {
                assert!(!nsjail.success);
                assert!(nsjail.stderr.contains("Test error message"));
                println!("Only nsjail backend available for error test");
            }
            (None, Some(docker)) => {
                assert!(!docker.success);
                assert!(docker.stderr.contains("Test error message"));
                println!("Only Docker backend available for error test");
            }
            (None, None) => {
                println!("No backends available for error testing");
            }
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    async fn measure_execution_time(backend_type: SandboxBackendType, iterations: usize) -> Option<Vec<u128>> {
        let backend = create_backend(backend_type);
        
        if let Ok(backend) = backend {
            if backend.is_available().await {
                let mut times = Vec::new();
                
                for i in 0..iterations {
                    let request = SandboxRequest {
                        id: Uuid::new_v4().to_string(),
                        runtime: "node".to_string(),
                        code: format!("console.log('Iteration {}');", i),
                        entry_point: None,
                        timeout_ms: 5000,
                        memory_limit_mb: 128,
                        env_vars: HashMap::new(),
                    };
                    
                    let start = Instant::now();
                    
                    if backend.create_sandbox(&request).await.is_ok() {
                        if let Ok(_) = backend.execute_sandbox(&request).await {
                            let duration = start.elapsed().as_millis();
                            times.push(duration);
                        }
                        let _ = backend.cleanup_sandbox(&request.id).await;
                    }
                }
                
                return Some(times);
            }
        }
        None
    }

    #[tokio::test]
    async fn test_performance_comparison() {
        let iterations = 5;
        
        let nsjail_times = measure_execution_time(SandboxBackendType::Nsjail, iterations).await;
        let docker_times = measure_execution_time(SandboxBackendType::Docker, iterations).await;
        
        match (nsjail_times, docker_times) {
            (Some(nsjail), Some(docker)) => {
                let nsjail_avg = nsjail.iter().sum::<u128>() / nsjail.len() as u128;
                let docker_avg = docker.iter().sum::<u128>() / docker.len() as u128;
                
                println!("nsjail average execution time: {}ms", nsjail_avg);
                println!("Docker average execution time: {}ms", docker_avg);
                
                // Generally, nsjail should be faster
                if nsjail_avg < docker_avg {
                    println!("nsjail is faster than Docker (as expected)");
                } else {
                    println!("Docker performed better than expected");
                }
            }
            (Some(nsjail), None) => {
                let nsjail_avg = nsjail.iter().sum::<u128>() / nsjail.len() as u128;
                println!("nsjail average execution time: {}ms", nsjail_avg);
                println!("Docker not available for comparison");
            }
            (None, Some(docker)) => {
                let docker_avg = docker.iter().sum::<u128>() / docker.len() as u128;
                println!("Docker average execution time: {}ms", docker_avg);
                println!("nsjail not available for comparison");
            }
            (None, None) => {
                println!("No backends available for performance testing");
            }
        }
    }
}