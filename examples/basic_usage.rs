use serde_json::json;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:8070";

    println!("🚀 Sandbox Service Example");
    println!("========================");

    // Health check
    println!("\n1. Health Check");
    let health_response = client.get(&format!("{}/health", base_url)).send().await?;
    println!("Status: {}", health_response.status());
    println!("Response: {}", health_response.text().await?);

    // Create Node.js sandbox
    println!("\n2. Creating Node.js Sandbox");
    let create_request = json!({
        "runtime": "node",
        "code": "console.log('Hello from Node.js!'); console.log('Current time:', new Date().toISOString());",
        "timeout_ms": 5000,
        "memory_limit_mb": 128
    });

    let create_response = client
        .post(&format!("{}/sandbox", base_url))
        .json(&create_request)
        .send()
        .await?;

    let sandbox_info: serde_json::Value = create_response.json().await?;
    let sandbox_id = sandbox_info["id"].as_str().unwrap();
    
    println!("Created sandbox: {}", sandbox_id);
    println!("Sandbox info: {}", serde_json::to_string_pretty(&sandbox_info)?);

    // Execute the sandbox
    println!("\n3. Executing Node.js Code");
    let execute_response = client
        .post(&format!("{}/sandbox/{}/execute", base_url, sandbox_id))
        .send()
        .await?;

    let execution_result: serde_json::Value = execute_response.json().await?;
    println!("Execution result: {}", serde_json::to_string_pretty(&execution_result)?);

    // Create TypeScript sandbox
    println!("\n4. Creating TypeScript Sandbox");
    let ts_create_request = json!({
        "runtime": "typescript",
        "code": "interface User { name: string; age: number; } const user: User = { name: 'Alice', age: 30 }; console.log(`Hello, ${user.name}! You are ${user.age} years old.`);",
        "timeout_ms": 10000,
        "memory_limit_mb": 256
    });

    let ts_create_response = client
        .post(&format!("{}/sandbox", base_url))
        .json(&ts_create_request)
        .send()
        .await?;

    let ts_sandbox_info: serde_json::Value = ts_create_response.json().await?;
    let ts_sandbox_id = ts_sandbox_info["id"].as_str().unwrap();
    
    println!("Created TypeScript sandbox: {}", ts_sandbox_id);

    // Execute TypeScript code
    println!("\n5. Executing TypeScript Code");
    let ts_execute_response = client
        .post(&format!("{}/sandbox/{}/execute", base_url, ts_sandbox_id))
        .send()
        .await?;

    let ts_execution_result: serde_json::Value = ts_execute_response.json().await?;
    println!("TypeScript execution result: {}", serde_json::to_string_pretty(&ts_execution_result)?);

    // Create Bun sandbox
    println!("\n6. Creating Bun Sandbox");
    let bun_create_request = json!({
        "runtime": "bun",
        "code": "console.log('Hello from Bun!'); console.log('Bun version:', Bun.version); const data = { message: 'Fast JavaScript runtime' }; console.log(JSON.stringify(data, null, 2));",
        "timeout_ms": 5000,
        "memory_limit_mb": 128
    });

    let bun_create_response = client
        .post(&format!("{}/sandbox", base_url))
        .json(&bun_create_request)
        .send()
        .await?;

    let bun_sandbox_info: serde_json::Value = bun_create_response.json().await?;
    let bun_sandbox_id = bun_sandbox_info["id"].as_str().unwrap();
    
    println!("Created Bun sandbox: {}", bun_sandbox_id);

    // Execute Bun code
    println!("\n7. Executing Bun Code");
    let bun_execute_response = client
        .post(&format!("{}/sandbox/{}/execute", base_url, bun_sandbox_id))
        .send()
        .await?;

    let bun_execution_result: serde_json::Value = bun_execute_response.json().await?;
    println!("Bun execution result: {}", serde_json::to_string_pretty(&bun_execution_result)?);

    // List all sandboxes
    println!("\n8. Listing All Sandboxes");
    let list_response = client.get(&format!("{}/sandbox", base_url)).send().await?;
    let sandboxes: serde_json::Value = list_response.json().await?;
    println!("All sandboxes: {}", serde_json::to_string_pretty(&sandboxes)?);

    // Clean up sandboxes
    println!("\n9. Cleaning Up Sandboxes");
    for sandbox_id in [sandbox_id, ts_sandbox_id, bun_sandbox_id] {
        let delete_response = client
            .delete(&format!("{}/sandbox/{}", base_url, sandbox_id))
            .send()
            .await?;
        println!("Deleted sandbox {}: {}", sandbox_id, delete_response.status());
    }

    // Demonstrate error handling
    println!("\n10. Error Handling Example");
    let error_request = json!({
        "runtime": "node",
        "code": "throw new Error('This is a test error'); console.log('This will not be executed');",
        "timeout_ms": 5000,
        "memory_limit_mb": 128
    });

    let error_create_response = client
        .post(&format!("{}/sandbox", base_url))
        .json(&error_request)
        .send()
        .await?;

    let error_sandbox_info: serde_json::Value = error_create_response.json().await?;
    let error_sandbox_id = error_sandbox_info["id"].as_str().unwrap();

    let error_execute_response = client
        .post(&format!("{}/sandbox/{}/execute", base_url, error_sandbox_id))
        .send()
        .await?;

    let error_result: serde_json::Value = error_execute_response.json().await?;
    println!("Error handling result: {}", serde_json::to_string_pretty(&error_result)?);

    // Clean up error sandbox
    client
        .delete(&format!("{}/sandbox/{}", base_url, error_sandbox_id))
        .send()
        .await?;

    println!("\n✅ Example completed successfully!");
    Ok(())
}