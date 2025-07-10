use std::collections::HashMap;
use std::sync::Arc;
use axum::{
    extract::{Path, State, Request},
    http::{StatusCode, Uri},
    response::Response,
    routing::any,
    Router,
};
use tokio::sync::RwLock;
use tracing::{error, info};

#[cfg(feature = "docker")]
use bollard::Docker;

/// Port allocation manager for sandbox containers
#[derive(Debug, Clone)]
pub struct PortAllocator {
    next_port: Arc<RwLock<u16>>,
    allocated_ports: Arc<RwLock<HashMap<String, u16>>>,
}

impl PortAllocator {
    pub fn new(start_port: u16) -> Self {
        Self {
            next_port: Arc::new(RwLock::new(start_port)),
            allocated_ports: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn allocate_port(&self, sandbox_id: &str) -> u16 {
        let mut next_port = self.next_port.write().await;
        let mut allocated = self.allocated_ports.write().await;
        
        // Check if already allocated
        if let Some(&port) = allocated.get(sandbox_id) {
            return port;
        }
        
        // Find next available port
        while allocated.values().any(|&p| p == *next_port) {
            *next_port += 1;
        }
        
        let port = *next_port;
        allocated.insert(sandbox_id.to_string(), port);
        *next_port += 1;
        
        info!("Allocated port {} for sandbox {}", port, sandbox_id);
        port
    }
    
    pub async fn deallocate_port(&self, sandbox_id: &str) {
        let mut allocated = self.allocated_ports.write().await;
        if let Some(port) = allocated.remove(sandbox_id) {
            info!("Deallocated port {} for sandbox {}", port, sandbox_id);
        }
    }
    
    pub async fn get_port(&self, sandbox_id: &str) -> Option<u16> {
        let allocated = self.allocated_ports.read().await;
        allocated.get(sandbox_id).copied()
    }
}

/// Reverse proxy state
#[derive(Clone)]
pub struct ProxyState {
    pub client: reqwest::Client,
    pub port_allocator: PortAllocator,
}

impl ProxyState {
    pub fn new(start_port: u16) -> Self {
        Self {
            client: reqwest::Client::new(),
            port_allocator: PortAllocator::new(start_port),
        }
    }
}

/// Get the mapped port for a container by inspecting Docker
async fn get_container_port(container_id: &str) -> Option<u16> {
    #[cfg(feature = "docker")]
    {
        let docker = Docker::connect_with_local_defaults().ok()?;
        let container_info = docker.inspect_container(container_id, None).await.ok()?;
        
        if let Some(network_settings) = container_info.network_settings {
            if let Some(ports) = network_settings.ports {
                // Look for port 3000/tcp mapping
                if let Some(port_bindings) = ports.get("3000/tcp") {
                    if let Some(bindings) = port_bindings {
                        if let Some(binding) = bindings.first() {
                            if let Some(host_port) = &binding.host_port {
                                return host_port.parse::<u16>().ok();
                            }
                        }
                    }
                }
            }
        }
    }
    
    #[cfg(not(feature = "docker"))]
    {
        let _ = container_id; // Suppress unused warning
    }
    
    None
}

/// Proxy handler for sandbox web services
pub async fn proxy_handler(
    Path((sandbox_id, remainder)): Path<(String, String)>,
    State(state): State<ProxyState>,
    req: Request,
) -> Result<Response, StatusCode> {
    // Try to get port from port allocator first
    let port = if let Some(port) = state.port_allocator.get_port(&sandbox_id).await {
        port
    } else {
        // Fallback: inspect Docker container to find mapped port
        get_container_port(&sandbox_id).await
            .ok_or(StatusCode::NOT_FOUND)?
    };

    // Build the target URL - strip the proxy prefix and use the remainder
    let target_path = if remainder.is_empty() { 
        "/" 
    } else { 
        if remainder.starts_with('/') { &remainder } else { &format!("/{}", remainder) }
    };
    let query = req.uri().query().map(|q| format!("?{}", q)).unwrap_or_default();
    
    let target_url = format!("http://127.0.0.1:{}{}{}", port, target_path, query);
    
    // Forward the request using reqwest
    let method = req.method().clone();
    let headers = req.headers().clone();
    let body = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let method_str = method.as_str();
    let mut request_builder = state.client.request(reqwest::Method::from_bytes(method_str.as_bytes()).unwrap(), &target_url);
    
    // Copy headers (convert from axum to reqwest)
    for (name, value) in headers {
        if let Some(name) = name {
            if let Ok(value_str) = value.to_str() {
                request_builder = request_builder.header(name.as_str(), value_str);
            }
        }
    }
    
    // Send the request
    let response = request_builder
        .body(body)
        .send()
        .await
        .map_err(|e| {
            error!("Proxy request failed: {}", e);
            StatusCode::BAD_GATEWAY
        })?;
    
    // Build the response
    let mut response_builder = Response::builder()
        .status(response.status().as_u16());
    
    // Copy response headers (convert from reqwest to axum)
    for (name, value) in response.headers() {
        if let Ok(value_str) = value.to_str() {
            response_builder = response_builder.header(name.as_str(), value_str);
        }
    }
    
    let body = response.bytes().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    response_builder
        .body(axum::body::Body::from(body))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Proxy handler for sandbox web services (no trailing path)
pub async fn proxy_handler_root(
    Path(sandbox_id): Path<String>,
    State(state): State<ProxyState>,
    req: Request,
) -> Result<Response, StatusCode> {
    // Try to get port from port allocator first
    let port = if let Some(port) = state.port_allocator.get_port(&sandbox_id).await {
        port
    } else {
        // Fallback: inspect Docker container to find mapped port
        get_container_port(&sandbox_id).await
            .ok_or(StatusCode::NOT_FOUND)?
    };

    // Build the target URL - default to root path
    let query = req.uri().query().map(|q| format!("?{}", q)).unwrap_or_default();
    let target_url = format!("http://127.0.0.1:{}/{}", port, query);
    
    // Forward the request using reqwest
    let method = req.method().clone();
    let headers = req.headers().clone();
    let body = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let method_str = method.as_str();
    let mut request_builder = state.client.request(reqwest::Method::from_bytes(method_str.as_bytes()).unwrap(), &target_url);
    
    // Copy headers (convert from axum to reqwest)
    for (name, value) in headers {
        if let Some(name) = name {
            if let Ok(value_str) = value.to_str() {
                request_builder = request_builder.header(name.as_str(), value_str);
            }
        }
    }
    
    // Send the request
    let response = request_builder
        .body(body)
        .send()
        .await
        .map_err(|e| {
            error!("Proxy request failed: {}", e);
            StatusCode::BAD_GATEWAY
        })?;
    
    // Build the response
    let mut response_builder = Response::builder()
        .status(response.status().as_u16());
    
    // Copy response headers (convert from reqwest to axum)
    for (name, value) in response.headers() {
        if let Ok(value_str) = value.to_str() {
            response_builder = response_builder.header(name.as_str(), value_str);
        }
    }
    
    let body = response.bytes().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    response_builder
        .body(axum::body::Body::from(body))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Unified proxy handler that handles both root and path requests
pub async fn unified_proxy_handler(
    path: axum::extract::Path<String>,
    State(state): State<ProxyState>,
    req: Request,
) -> Result<Response, StatusCode> {
    let full_path = path.0;
    
    // Extract sandbox_id and remainder from the full path
    let path_parts: Vec<&str> = full_path.split('/').filter(|s| !s.is_empty()).collect();
    
    if path_parts.len() < 2 || path_parts[0] != "proxy" {
        return Err(StatusCode::NOT_FOUND);
    }
    
    let sandbox_id = path_parts[1];
    let remainder_parts = &path_parts[2..];
    let target_path = if remainder_parts.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", remainder_parts.join("/"))
    };
    
    // Try to get port from port allocator first
    let port = if let Some(port) = state.port_allocator.get_port(sandbox_id).await {
        port
    } else {
        // Fallback: inspect Docker container to find mapped port
        get_container_port(sandbox_id).await
            .ok_or(StatusCode::NOT_FOUND)?
    };

    // Build the target URL
    let query = req.uri().query().map(|q| format!("?{}", q)).unwrap_or_default();
    let target_url = format!("http://127.0.0.1:{}{}{}", port, target_path, query);
    
    // Forward the request using reqwest
    let method = req.method().clone();
    let headers = req.headers().clone();
    let body = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let method_str = method.as_str();
    let mut request_builder = state.client.request(reqwest::Method::from_bytes(method_str.as_bytes()).unwrap(), &target_url);
    
    // Copy headers (convert from axum to reqwest)
    for (name, value) in headers {
        if let Some(name) = name {
            if let Ok(value_str) = value.to_str() {
                request_builder = request_builder.header(name.as_str(), value_str);
            }
        }
    }
    
    // Send the request
    let response = request_builder
        .body(body)
        .send()
        .await
        .map_err(|e| {
            error!("Proxy request failed: {}", e);
            StatusCode::BAD_GATEWAY
        })?;
    
    // Build the response
    let mut response_builder = Response::builder()
        .status(response.status().as_u16());
    
    // Copy response headers (convert from reqwest to axum)
    for (name, value) in response.headers() {
        if let Ok(value_str) = value.to_str() {
            response_builder = response_builder.header(name.as_str(), value_str);
        }
    }
    
    let body = response.bytes().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    response_builder
        .body(axum::body::Body::from(body))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Create the proxy router
pub fn create_proxy_router(state: ProxyState) -> Router {
    Router::new()
        .route("/proxy/*path", any(unified_proxy_handler))
        .with_state(state)
}