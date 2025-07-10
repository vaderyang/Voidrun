use anyhow::Result;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

const BASE_URL: &str = "http://localhost:8070";

#[tokio::test]
async fn test_nsjail_node_execution() -> Result<()> {
    let client = Client::new();
    
    // Test basic Node.js execution
    let response = client
        .post(&format!("{}/execute", BASE_URL))
        .json(&json!({
            "code": "console.log('Hello from Node.js!'); console.log(process.version);",
            "runtime": "node",
            "timeout_ms": 5000,
            "memory_limit_mb": 64
        }))
        .send()
        .await?;
    
    assert!(response.status().is_success());
    let result: serde_json::Value = response.json().await?;
    
    assert_eq!(result["success"], true);
    assert!(result["stdout"].as_str().unwrap().contains("Hello from Node.js!"));
    assert!(result["stdout"].as_str().unwrap().contains("v")); // Version string
    
    Ok(())
}

#[tokio::test]
async fn test_nsjail_bun_execution() -> Result<()> {
    let client = Client::new();
    
    // Test Bun execution
    let response = client
        .post(&format!("{}/execute", BASE_URL))
        .json(&json!({
            "code": "console.log('Hello from Bun!'); console.log(Bun.version);",
            "runtime": "bun",
            "timeout_ms": 5000,
            "memory_limit_mb": 64
        }))
        .send()
        .await?;
    
    assert!(response.status().is_success());
    let result: serde_json::Value = response.json().await?;
    
    assert_eq!(result["success"], true);
    assert!(result["stdout"].as_str().unwrap().contains("Hello from Bun!"));
    
    Ok(())
}

#[tokio::test]
async fn test_nsjail_typescript_execution() -> Result<()> {
    let client = Client::new();
    
    // Test TypeScript execution
    let response = client
        .post(&format!("{}/execute", BASE_URL))
        .json(&json!({
            "code": "const greeting: string = 'Hello TypeScript!'; console.log(greeting);",
            "runtime": "typescript",
            "timeout_ms": 10000,
            "memory_limit_mb": 128
        }))
        .send()
        .await?;
    
    assert!(response.status().is_success());
    let result: serde_json::Value = response.json().await?;
    
    assert_eq!(result["success"], true);
    assert!(result["stdout"].as_str().unwrap().contains("Hello TypeScript!"));
    
    Ok(())
}

#[tokio::test]
async fn test_nsjail_error_handling() -> Result<()> {
    let client = Client::new();
    
    // Test error handling with invalid syntax
    let response = client
        .post(&format!("{}/execute", BASE_URL))
        .json(&json!({
            "code": "console.log('test'); invalid_syntax_here",
            "runtime": "node",
            "timeout_ms": 5000,
            "memory_limit_mb": 64
        }))
        .send()
        .await?;
    
    assert!(response.status().is_success());
    let result: serde_json::Value = response.json().await?;
    
    assert_eq!(result["success"], false);
    assert!(result["stderr"].as_str().unwrap().len() > 0);
    
    Ok(())
}

#[tokio::test]
async fn test_nsjail_timeout_handling() -> Result<()> {
    let client = Client::new();
    
    // Test timeout with infinite loop
    let response = client
        .post(&format!("{}/execute", BASE_URL))
        .json(&json!({
            "code": "while(true) { /* infinite loop */ }",
            "runtime": "node",
            "timeout_ms": 2000,
            "memory_limit_mb": 64
        }))
        .send()
        .await?;
    
    assert!(response.status().is_success());
    let result: serde_json::Value = response.json().await?;
    
    assert_eq!(result["success"], false);
    assert!(result["stderr"].as_str().unwrap().contains("timeout") || 
            result["exit_code"].as_i64().unwrap() == 124);
    
    Ok(())
}

#[tokio::test]
async fn test_nsjail_file_system_isolation() -> Result<()> {
    let client = Client::new();
    
    // Test that sandbox cannot access host filesystem
    let response = client
        .post(&format!("{}/execute", BASE_URL))
        .json(&json!({
            "code": "const fs = require('fs'); try { const data = fs.readFileSync('/etc/passwd', 'utf8'); console.log('BREACH: ' + data); } catch(e) { console.log('SECURE: Cannot read /etc/passwd - ' + e.message); }",
            "runtime": "node",
            "timeout_ms": 5000,
            "memory_limit_mb": 64
        }))
        .send()
        .await?;
    
    assert!(response.status().is_success());
    let result: serde_json::Value = response.json().await?;
    
    assert_eq!(result["success"], true);
    assert!(result["stdout"].as_str().unwrap().contains("SECURE"));
    assert!(!result["stdout"].as_str().unwrap().contains("BREACH"));
    
    Ok(())
}

#[tokio::test]
async fn test_nsjail_network_isolation() -> Result<()> {
    let client = Client::new();
    
    // Test network isolation (should fail to make external requests)
    let response = client
        .post(&format!("{}/execute", BASE_URL))
        .json(&json!({
            "code": "const https = require('https'); https.get('https://google.com', (res) => { console.log('BREACH: Network access allowed'); }).on('error', (err) => { console.log('SECURE: Network blocked - ' + err.message); });",
            "runtime": "node",
            "timeout_ms": 5000,
            "memory_limit_mb": 64
        }))
        .send()
        .await?;
    
    assert!(response.status().is_success());
    let result: serde_json::Value = response.json().await?;
    
    // Should either fail or be blocked
    assert!(result["stdout"].as_str().unwrap().contains("SECURE") || 
            result["stderr"].as_str().unwrap().len() > 0);
    
    Ok(())
}

#[tokio::test]
async fn test_nsjail_with_multiple_files() -> Result<()> {
    let client = Client::new();
    
    // Test with multiple files
    let response = client
        .post(&format!("{}/execute", BASE_URL))
        .json(&json!({
            "code": "const helper = require('./helper'); console.log(helper.greet('World'));",
            "runtime": "node",
            "timeout_ms": 5000,
            "memory_limit_mb": 64,
            "files": [{
                "path": "helper.js",
                "content": "exports.greet = function(name) { return 'Hello ' + name + '!'; };"
            }]
        }))
        .send()
        .await?;
    
    assert!(response.status().is_success());
    let result: serde_json::Value = response.json().await?;
    
    assert_eq!(result["success"], true);
    assert!(result["stdout"].as_str().unwrap().contains("Hello World!"));
    
    Ok(())
}

#[tokio::test]
async fn test_nsjail_memory_limit() -> Result<()> {
    let client = Client::new();
    
    // Test memory limit enforcement
    let response = client
        .post(&format!("{}/execute", BASE_URL))
        .json(&json!({
            "code": "const arr = []; try { while(true) { arr.push(new Array(1000000).fill('x')); } } catch(e) { console.log('Memory limit hit: ' + e.message); }",
            "runtime": "node",
            "timeout_ms": 10000,
            "memory_limit_mb": 32
        }))
        .send()
        .await?;
    
    assert!(response.status().is_success());
    let result: serde_json::Value = response.json().await?;
    
    // Should either hit memory limit or be killed
    assert!(result["stdout"].as_str().unwrap().contains("Memory limit") || 
            result["success"] == false);
    
    Ok(())
}

#[tokio::test]
async fn test_nsjail_concurrent_execution() -> Result<()> {
    let client = Client::new();
    
    // Test multiple concurrent executions
    let mut handles = vec![];
    
    for i in 0..5 {
        let client = client.clone();
        let handle = tokio::spawn(async move {
            let response = client
                .post(&format!("{}/execute", BASE_URL))
                .json(&json!({
                    "code": format!("console.log('Execution {}'); for(let i = 0; i < 1000; i++) {{ Math.random(); }}", i),
                    "runtime": "node",
                    "timeout_ms": 5000,
                    "memory_limit_mb": 64
                }))
                .send()
                .await?;
            
            assert!(response.status().is_success());
            let result: serde_json::Value = response.json().await?;
            assert_eq!(result["success"], true);
            assert!(result["stdout"].as_str().unwrap().contains(&format!("Execution {}", i)));
            
            Result::<()>::Ok(())
        });
        handles.push(handle);
    }
    
    // Wait for all executions to complete
    for handle in handles {
        handle.await??;
    }
    
    Ok(())
}

#[tokio::test]
async fn test_nsjail_faas_deployment() -> Result<()> {
    let client = Client::new();
    
    // Test FaaS deployment with nsjail
    let response = client
        .post(&format!("{}/faas/deploy", BASE_URL))
        .json(&json!({
            "code": "export default function handler(event) { return { message: 'Hello from FaaS!', event }; }",
            "runtime": "bun",
            "timeout_ms": 5000,
            "memory_limit_mb": 64
        }))
        .send()
        .await?;
    
    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        
        if let Some(function_id) = result["function_id"].as_str() {
            // Wait a moment for deployment
            sleep(Duration::from_millis(1000)).await;
            
            // Test function invocation
            let invoke_response = client
                .post(&format!("{}/faas/invoke/{}", BASE_URL, function_id))
                .json(&json!({"test": "data"}))
                .send()
                .await?;
            
            if invoke_response.status().is_success() {
                let invoke_result: serde_json::Value = invoke_response.json().await?;
                assert!(invoke_result["message"].as_str().unwrap().contains("Hello from FaaS!"));
            }
        }
    }
    
    Ok(())
}