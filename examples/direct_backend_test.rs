use sandbox_service::sandbox::backend::{create_backend, SandboxBackendType};
use sandbox_service::sandbox::{SandboxRequest, SandboxResponse};
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Docker Backend Directly");
    
    // Test Docker backend directly
    let backend = create_backend(SandboxBackendType::Docker)?;
    
    if !backend.is_available().await {
        println!("❌ Docker backend is not available");
        return Ok(());
    }
    
    println!("✅ Docker backend is available");
    
    // Create a test request
    let request = SandboxRequest {
        id: Uuid::new_v4().to_string(),
        runtime: "node".to_string(),
        code: "console.log('Hello from direct test!'); console.log('Success!');".to_string(),
        entry_point: None,
        timeout_ms: 5000,
        memory_limit_mb: 128,
        env_vars: HashMap::new(),
    };
    
    println!("🔨 Creating sandbox: {}", request.id);
    
    // Create sandbox
    match backend.create_sandbox(&request).await {
        Ok(_) => println!("✅ Sandbox created successfully"),
        Err(e) => {
            println!("❌ Failed to create sandbox: {}", e);
            return Ok(());
        }
    }
    
    println!("🚀 Executing sandbox...");
    
    // Execute sandbox
    match backend.execute_sandbox(&request).await {
        Ok(response) => {
            println!("✅ Sandbox executed successfully!");
            println!("   Success: {}", response.success);
            println!("   Exit code: {:?}", response.exit_code);
            println!("   Execution time: {}ms", response.execution_time_ms);
            println!("   Stdout: {}", response.stdout);
            if !response.stderr.is_empty() {
                println!("   Stderr: {}", response.stderr);
            }
        }
        Err(e) => {
            println!("❌ Failed to execute sandbox: {}", e);
            return Ok(());
        }
    }
    
    println!("🧹 Cleaning up sandbox...");
    
    // Cleanup
    match backend.cleanup_sandbox(&request.id).await {
        Ok(_) => println!("✅ Sandbox cleaned up successfully"),
        Err(e) => println!("⚠️  Failed to cleanup sandbox: {}", e),
    }
    
    println!("🎉 Direct backend test completed successfully!");
    Ok(())
}