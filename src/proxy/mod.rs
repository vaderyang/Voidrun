use std::collections::HashMap;
use std::sync::Arc;
use axum::{
    extract::{Path, State, Request},
    http::StatusCode,
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
    allocated_ports: Arc<RwLock<HashMap<String, u16>>>,
}

impl PortAllocator {
    pub fn new(_start_port: u16) -> Self {
        Self {
            allocated_ports: Arc::new(RwLock::new(HashMap::new())),
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
    pub faas_manager: Option<Arc<crate::faas::FaasManager>>,
}

impl ProxyState {
    pub fn new(start_port: u16) -> Self {
        Self {
            client: reqwest::Client::new(),
            port_allocator: PortAllocator::new(start_port),
            faas_manager: None,
        }
    }
    
    pub fn with_faas_manager(mut self, faas_manager: Arc<crate::faas::FaasManager>) -> Self {
        self.faas_manager = Some(faas_manager);
        self
    }
}

/// Get the mapped port for a container by inspecting Docker
async fn get_container_port(sandbox_id: &str) -> Option<u16> {
    #[cfg(feature = "docker")]
    {
        info!("[PROXY] Looking up container port for sandbox {}", sandbox_id);
        let docker = Docker::connect_with_local_defaults().ok()?;
        
        // First, find the container by name (sandbox_id is used as container name)
        let containers = docker.list_containers(Some(bollard::container::ListContainersOptions {
            all: true,
            filters: [("name".to_string(), vec![sandbox_id.to_string()])].into(),
            ..Default::default()
        })).await.ok()?;
        
        if containers.is_empty() {
            info!("[PROXY] No container found with name {}", sandbox_id);
            return None;
        }
        
        let container = containers.first()?;
        let container_id = &container.id.as_ref()?;
        info!("[PROXY] Found container {} for sandbox {}", container_id, sandbox_id);
        
        let container_info = docker.inspect_container(container_id, None).await.ok()?;
        
        if let Some(network_settings) = container_info.network_settings {
            if let Some(ports) = network_settings.ports {
                info!("[PROXY] Container ports available: {:?}", ports.keys().collect::<Vec<_>>());
                // Look for port 3000/tcp mapping
                if let Some(port_bindings) = ports.get("3000/tcp") {
                    if let Some(bindings) = port_bindings {
                        if let Some(binding) = bindings.first() {
                            if let Some(host_port) = &binding.host_port {
                                let port = host_port.parse::<u16>().ok()?;
                                info!("[PROXY] Found host port {} mapped to container port 3000", port);
                                return Some(port);
                            }
                        }
                    }
                }
                info!("[PROXY] No port 3000/tcp mapping found for container");
            } else {
                info!("[PROXY] No port mappings found for container");
            }
        } else {
            info!("[PROXY] No network settings found for container");
        }
        
        info!("[PROXY] Failed to find port mapping for sandbox {}", sandbox_id);
    }
    
    #[cfg(not(feature = "docker"))]
    {
        let _ = sandbox_id; // Suppress unused warning
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
    let target_url = format!("http://127.0.0.1:{}{}", port, query);
    
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
        .route("/proxy/:sandbox_id", any(proxy_handler_root))
        .route("/proxy/:sandbox_id/*remainder", any(proxy_handler))
        .route("/faas/:deployment_id", any(faas_proxy_handler_root))
        .route("/faas/:deployment_id/*remainder", any(faas_proxy_handler))
        .with_state(state)
}

/// FaaS proxy handler for root path
pub async fn faas_proxy_handler_root(
    Path(deployment_id): Path<String>,
    State(state): State<ProxyState>,
    req: Request,
) -> Result<Response, StatusCode> {
    info!("[PROXY] FaaS root request - Deployment: {}", deployment_id);
    
    // Get sandbox ID from FaaS manager
    let sandbox_id = if let Some(ref faas_manager) = state.faas_manager {
        match faas_manager.get_deployment_for_proxy(&deployment_id).await {
            Some(id) => {
                info!("[PROXY] Found sandbox {} for deployment {}", id, deployment_id);
                id
            }
            None => {
                error!("[PROXY] Deployment {} not found", deployment_id);
                return Err(StatusCode::NOT_FOUND);
            }
        }
    } else {
        error!("[PROXY] FaaS manager not available");
        return Err(StatusCode::NOT_FOUND);
    };

    // Get port
    let port = if let Some(port) = state.port_allocator.get_port(&sandbox_id).await {
        info!("[PROXY] Using allocated port {} for sandbox {}", port, sandbox_id);
        port
    } else {
        info!("[PROXY] No allocated port for sandbox {}, checking container", sandbox_id);
        match get_container_port(&sandbox_id).await {
            Some(port) => {
                info!("[PROXY] Found container port {} for sandbox {}", port, sandbox_id);
                port
            }
            None => {
                error!("[PROXY] No port found for sandbox {}", sandbox_id);
                return Err(StatusCode::NOT_FOUND);
            }
        }
    };

    // Build target URL
    let query = req.uri().query().map(|q| format!("?{}", q)).unwrap_or_default();
    let target_url = format!("http://127.0.0.1:{}{}", port, query);
    
    info!("[PROXY] Forwarding root to: {}", target_url);
    forward_request(state, req, target_url).await
}

/// FaaS proxy handler with path
pub async fn faas_proxy_handler(
    Path((deployment_id, remainder)): Path<(String, String)>,
    State(state): State<ProxyState>,
    req: Request,
) -> Result<Response, StatusCode> {
    info!("[PROXY] FaaS request - Deployment: {}, Path: {}", deployment_id, remainder);
    
    // Get sandbox ID from FaaS manager
    let sandbox_id = if let Some(ref faas_manager) = state.faas_manager {
        match faas_manager.get_deployment_for_proxy(&deployment_id).await {
            Some(id) => {
                info!("[PROXY] Found sandbox {} for deployment {}", id, deployment_id);
                id
            }
            None => {
                error!("[PROXY] Deployment {} not found", deployment_id);
                return Err(StatusCode::NOT_FOUND);
            }
        }
    } else {
        error!("[PROXY] FaaS manager not available");
        return Err(StatusCode::NOT_FOUND);
    };

    // Get port
    let port = if let Some(port) = state.port_allocator.get_port(&sandbox_id).await {
        info!("[PROXY] Using allocated port {} for sandbox {}", port, sandbox_id);
        port
    } else {
        info!("[PROXY] No allocated port for sandbox {}, checking container", sandbox_id);
        match get_container_port(&sandbox_id).await {
            Some(port) => {
                info!("[PROXY] Found container port {} for sandbox {}", port, sandbox_id);
                port
            }
            None => {
                error!("[PROXY] No port found for sandbox {}", sandbox_id);
                return Err(StatusCode::NOT_FOUND);
            }
        }
    };

    // Build target URL
    let target_path = if remainder.starts_with('/') { &remainder } else { &format!("/{}", remainder) };
    let query = req.uri().query().map(|q| format!("?{}", q)).unwrap_or_default();
    let target_url = format!("http://127.0.0.1:{}{}{}", port, target_path, query);
    
    info!("[PROXY] Forwarding to: {}", target_url);
    forward_request(state, req, target_url).await
}

/// Helper function to forward requests
async fn forward_request(
    state: ProxyState,
    req: Request,
    target_url: String,
) -> Result<Response, StatusCode> {
    let method = req.method().clone();
    let headers = req.headers().clone();
    let body = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let method_str = method.as_str();
    let mut request_builder = state.client.request(
        reqwest::Method::from_bytes(method_str.as_bytes()).unwrap(), 
        &target_url
    );
    
    // Copy headers
    for (name, value) in headers {
        if let Some(name) = name {
            if let Ok(value_str) = value.to_str() {
                request_builder = request_builder.header(name.as_str(), value_str);
            }
        }
    }
    
    // Send request
    let response = request_builder
        .body(body)
        .send()
        .await
        .map_err(|e| {
            error!("Proxy request failed: {}", e);
            StatusCode::BAD_GATEWAY
        })?;
    
    // Build response
    let mut response_builder = Response::builder()
        .status(response.status().as_u16());
    
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