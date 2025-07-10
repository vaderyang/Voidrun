use axum::http::StatusCode;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;

use sandbox_service::api::{create_router, CreateSandboxRequest, ExecutionResult, SandboxInfo};
use sandbox_service::sandbox::backend::SandboxBackendType;
use sandbox_service::sandbox::manager::SandboxManager;

async fn create_test_app() -> axum::Router {
    let backend_type = if std::env::var("TEST_BACKEND").as_deref() == Ok("docker") {
        SandboxBackendType::Docker
    } else {
        SandboxBackendType::Nsjail
    };

    let manager = SandboxManager::new(backend_type).await.unwrap();
    let app_state = Arc::new(RwLock::new(manager));
    create_router(app_state)
}

async fn make_request<T>(
    app: &axum::Router,
    method: &str,
    path: &str,
    body: Option<T>,
) -> (StatusCode, Value)
where
    T: serde::Serialize,
{
    use axum::body::Body;
    use axum::http::{Method, Request};
    use tower::ServiceExt;

    let mut request_builder = Request::builder().method(method).uri(path);

    let body = if let Some(body) = body {
        request_builder = request_builder.header("content-type", "application/json");
        Body::from(serde_json::to_string(&body).unwrap())
    } else {
        Body::empty()
    };

    let request = request_builder.body(body).unwrap();
    let response = app.clone().oneshot(request).await.unwrap();

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap_or(json!({}));

    (status, json)
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_app().await;
    let (status, body) = make_request::<()>(&app, "GET", "/health", None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
    assert_eq!(body["service"], "sandbox-service");
    assert_eq!(body["version"], "0.1.0");
}

#[tokio::test]
async fn test_create_node_sandbox() {
    let app = create_test_app().await;
    
    let request = CreateSandboxRequest {
        runtime: "node".to_string(),
        code: "console.log('Hello, Node.js!');".to_string(),
        entry_point: None,
        timeout_ms: Some(5000),
        memory_limit_mb: Some(128),
        env_vars: None,
    };

    let (status, body) = make_request(&app, "POST", "/sandbox", Some(request)).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["id"].is_string());
    assert_eq!(body["runtime"], "node");
    assert_eq!(body["status"], "created");
    assert_eq!(body["timeout_ms"], 5000);
    assert_eq!(body["memory_limit_mb"], 128);
}

#[tokio::test]
async fn test_create_and_execute_sandbox() {
    let app = create_test_app().await;
    
    let request = CreateSandboxRequest {
        runtime: "node".to_string(),
        code: "console.log('Hello, World!'); console.log('Second line');".to_string(),
        entry_point: None,
        timeout_ms: Some(5000),
        memory_limit_mb: Some(128),
        env_vars: None,
    };

    let (status, body) = make_request(&app, "POST", "/sandbox", Some(request)).await;
    assert_eq!(status, StatusCode::OK);
    
    let sandbox_id = body["id"].as_str().unwrap();
    let execute_path = format!("/sandbox/{}/execute", sandbox_id);
    
    let (status, body) = make_request::<()>(&app, "POST", &execute_path, None).await;
    assert_eq!(status, StatusCode::OK);
    
    assert_eq!(body["sandbox_id"], sandbox_id);
    assert_eq!(body["success"], true);
    assert!(body["stdout"].as_str().unwrap().contains("Hello, World!"));
    assert!(body["stdout"].as_str().unwrap().contains("Second line"));
    assert_eq!(body["stderr"], "");
    assert_eq!(body["exit_code"], 0);
    assert!(body["execution_time_ms"].as_u64().unwrap() > 0);
}

#[tokio::test]
async fn test_typescript_sandbox() {
    let app = create_test_app().await;
    
    let request = CreateSandboxRequest {
        runtime: "typescript".to_string(),
        code: "interface User { name: string; age: number; } const user: User = { name: 'Alice', age: 30 }; console.log(`Hello, ${user.name}!`);".to_string(),
        entry_point: None,
        timeout_ms: Some(10000),
        memory_limit_mb: Some(256),
        env_vars: None,
    };

    let (status, body) = make_request(&app, "POST", "/sandbox", Some(request)).await;
    assert_eq!(status, StatusCode::OK);
    
    let sandbox_id = body["id"].as_str().unwrap();
    let execute_path = format!("/sandbox/{}/execute", sandbox_id);
    
    let (status, body) = make_request::<()>(&app, "POST", &execute_path, None).await;
    assert_eq!(status, StatusCode::OK);
    
    assert_eq!(body["success"], true);
    assert!(body["stdout"].as_str().unwrap().contains("Hello, Alice!"));
}

#[tokio::test]
async fn test_error_handling() {
    let app = create_test_app().await;
    
    let request = CreateSandboxRequest {
        runtime: "node".to_string(),
        code: "throw new Error('Test error'); console.log('This should not run');".to_string(),
        entry_point: None,
        timeout_ms: Some(5000),
        memory_limit_mb: Some(128),
        env_vars: None,
    };

    let (status, body) = make_request(&app, "POST", "/sandbox", Some(request)).await;
    assert_eq!(status, StatusCode::OK);
    
    let sandbox_id = body["id"].as_str().unwrap();
    let execute_path = format!("/sandbox/{}/execute", sandbox_id);
    
    let (status, body) = make_request::<()>(&app, "POST", &execute_path, None).await;
    assert_eq!(status, StatusCode::OK);
    
    assert_eq!(body["success"], false);
    assert!(body["stderr"].as_str().unwrap().contains("Error: Test error"));
    assert_eq!(body["exit_code"], 1);
}

#[tokio::test]
async fn test_environment_variables() {
    let app = create_test_app().await;
    
    let mut env_vars = HashMap::new();
    env_vars.insert("TEST_VAR".to_string(), "test_value".to_string());
    env_vars.insert("NODE_ENV".to_string(), "sandbox".to_string());
    
    let request = CreateSandboxRequest {
        runtime: "node".to_string(),
        code: "console.log('TEST_VAR:', process.env.TEST_VAR); console.log('NODE_ENV:', process.env.NODE_ENV);".to_string(),
        entry_point: None,
        timeout_ms: Some(5000),
        memory_limit_mb: Some(128),
        env_vars: Some(env_vars),
    };

    let (status, body) = make_request(&app, "POST", "/sandbox", Some(request)).await;
    assert_eq!(status, StatusCode::OK);
    
    let sandbox_id = body["id"].as_str().unwrap();
    let execute_path = format!("/sandbox/{}/execute", sandbox_id);
    
    let (status, body) = make_request::<()>(&app, "POST", &execute_path, None).await;
    assert_eq!(status, StatusCode::OK);
    
    assert_eq!(body["success"], true);
    let stdout = body["stdout"].as_str().unwrap();
    assert!(stdout.contains("TEST_VAR: test_value"));
    assert!(stdout.contains("NODE_ENV: sandbox"));
}

#[tokio::test]
async fn test_get_sandbox_info() {
    let app = create_test_app().await;
    
    let request = CreateSandboxRequest {
        runtime: "node".to_string(),
        code: "console.log('test');".to_string(),
        entry_point: None,
        timeout_ms: Some(5000),
        memory_limit_mb: Some(128),
        env_vars: None,
    };

    let (status, body) = make_request(&app, "POST", "/sandbox", Some(request)).await;
    assert_eq!(status, StatusCode::OK);
    
    let sandbox_id = body["id"].as_str().unwrap();
    let info_path = format!("/sandbox/{}", sandbox_id);
    
    let (status, body) = make_request::<()>(&app, "GET", &info_path, None).await;
    assert_eq!(status, StatusCode::OK);
    
    assert_eq!(body["id"], sandbox_id);
    assert_eq!(body["runtime"], "node");
    assert_eq!(body["status"], "Created");
    assert_eq!(body["timeout_ms"], 5000);
    assert_eq!(body["memory_limit_mb"], 128);
    assert!(body["created_at"].is_string());
}

#[tokio::test]
async fn test_list_sandboxes() {
    let app = create_test_app().await;
    
    // Create multiple sandboxes
    for i in 0..3 {
        let request = CreateSandboxRequest {
            runtime: "node".to_string(),
            code: format!("console.log('Sandbox {}');", i),
            entry_point: None,
            timeout_ms: Some(5000),
            memory_limit_mb: Some(128),
            env_vars: None,
        };

        let (status, _) = make_request(&app, "POST", "/sandbox", Some(request)).await;
        assert_eq!(status, StatusCode::OK);
    }

    let (status, body) = make_request::<()>(&app, "GET", "/sandbox", None).await;
    assert_eq!(status, StatusCode::OK);
    
    let sandboxes = body.as_array().unwrap();
    assert!(sandboxes.len() >= 3);
    
    for sandbox in sandboxes {
        assert!(sandbox["id"].is_string());
        assert!(sandbox["runtime"].is_string());
        assert!(sandbox["status"].is_string());
        assert!(sandbox["created_at"].is_string());
    }
}

#[tokio::test]
async fn test_delete_sandbox() {
    let app = create_test_app().await;
    
    let request = CreateSandboxRequest {
        runtime: "node".to_string(),
        code: "console.log('test');".to_string(),
        entry_point: None,
        timeout_ms: Some(5000),
        memory_limit_mb: Some(128),
        env_vars: None,
    };

    let (status, body) = make_request(&app, "POST", "/sandbox", Some(request)).await;
    assert_eq!(status, StatusCode::OK);
    
    let sandbox_id = body["id"].as_str().unwrap();
    let delete_path = format!("/sandbox/{}", sandbox_id);
    
    let (status, _) = make_request::<()>(&app, "DELETE", &delete_path, None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    
    // Verify sandbox is deleted
    let (status, _) = make_request::<()>(&app, "GET", &delete_path, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_invalid_runtime() {
    let app = create_test_app().await;
    
    let request = CreateSandboxRequest {
        runtime: "python".to_string(),
        code: "print('Hello, Python!')".to_string(),
        entry_point: None,
        timeout_ms: Some(5000),
        memory_limit_mb: Some(128),
        env_vars: None,
    };

    let (status, body) = make_request(&app, "POST", "/sandbox", Some(request)).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_nonexistent_sandbox() {
    let app = create_test_app().await;
    
    let fake_id = "00000000-0000-0000-0000-000000000000";
    let path = format!("/sandbox/{}", fake_id);
    
    let (status, _) = make_request::<()>(&app, "GET", &path, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    
    let execute_path = format!("/sandbox/{}/execute", fake_id);
    let (status, _) = make_request::<()>(&app, "POST", &execute_path, None).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_timeout_handling() {
    let app = create_test_app().await;
    
    let request = CreateSandboxRequest {
        runtime: "node".to_string(),
        code: "while(true) { /* infinite loop */ }".to_string(),
        entry_point: None,
        timeout_ms: Some(1000), // 1 second timeout
        memory_limit_mb: Some(128),
        env_vars: None,
    };

    let (status, body) = make_request(&app, "POST", "/sandbox", Some(request)).await;
    assert_eq!(status, StatusCode::OK);
    
    let sandbox_id = body["id"].as_str().unwrap();
    let execute_path = format!("/sandbox/{}/execute", sandbox_id);
    
    let (status, body) = make_request::<()>(&app, "POST", &execute_path, None).await;
    assert_eq!(status, StatusCode::OK);
    
    assert_eq!(body["success"], false);
    assert!(body["stderr"].as_str().unwrap().contains("timed out"));
    assert_eq!(body["exit_code"], 124);
}

#[tokio::test]
async fn test_concurrent_execution() {
    let app = create_test_app().await;
    
    let mut handles = Vec::new();
    
    for i in 0..5 {
        let app_clone = app.clone();
        let handle = tokio::spawn(async move {
            let request = CreateSandboxRequest {
                runtime: "node".to_string(),
                code: format!("console.log('Concurrent execution {}'); console.log(Date.now());", i),
                entry_point: None,
                timeout_ms: Some(5000),
                memory_limit_mb: Some(128),
                env_vars: None,
            };

            let (status, body) = make_request(&app_clone, "POST", "/sandbox", Some(request)).await;
            assert_eq!(status, StatusCode::OK);
            
            let sandbox_id = body["id"].as_str().unwrap().to_string();
            let execute_path = format!("/sandbox/{}/execute", sandbox_id);
            
            let (status, body) = make_request::<()>(&app_clone, "POST", &execute_path, None).await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(body["success"], true);
            
            body["stdout"].as_str().unwrap().to_string()
        });
        handles.push(handle);
    }
    
    let results = futures::future::join_all(handles).await;
    
    for result in results {
        let stdout = result.unwrap();
        assert!(stdout.contains("Concurrent execution"));
    }
}

#[tokio::test]
async fn test_large_output() {
    let app = create_test_app().await;
    
    let request = CreateSandboxRequest {
        runtime: "node".to_string(),
        code: "for(let i = 0; i < 1000; i++) { console.log(`Line ${i}: This is a test of large output handling`); }".to_string(),
        entry_point: None,
        timeout_ms: Some(10000),
        memory_limit_mb: Some(256),
        env_vars: None,
    };

    let (status, body) = make_request(&app, "POST", "/sandbox", Some(request)).await;
    assert_eq!(status, StatusCode::OK);
    
    let sandbox_id = body["id"].as_str().unwrap();
    let execute_path = format!("/sandbox/{}/execute", sandbox_id);
    
    let (status, body) = make_request::<()>(&app, "POST", &execute_path, None).await;
    assert_eq!(status, StatusCode::OK);
    
    assert_eq!(body["success"], true);
    let stdout = body["stdout"].as_str().unwrap();
    assert!(stdout.contains("Line 0:"));
    assert!(stdout.contains("Line 999:"));
    assert!(stdout.len() > 50000); // Should be substantial output
}