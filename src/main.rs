use anyhow::Result;
use axum::Router;
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tower::ServiceBuilder;
use tracing::{info, warn};
use axum::{
    extract::ConnectInfo,
    http::Request,
    middleware::{self, Next},
    response::Response as AxumResponse,
};
use std::time::Instant;
use std::net::SocketAddr;

mod admin;
mod api;
mod config;
mod faas;
mod homepage;
mod proxy;
mod runtime;
mod sandbox;

use admin::create_admin_router;
use api::create_router;
use config::Config;
use faas::handlers::{FaasState, create_faas_router};
use homepage::homepage;
use proxy::{ProxyState, create_proxy_router};
use sandbox::manager::SandboxManager;

// Nginx-style access log middleware
async fn access_log_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request<axum::body::Body>,
    next: Next,
) -> AxumResponse {
    let start = Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();
    let version = req.version();
    let user_agent = req.headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("-")
        .to_string();
    let referer = req.headers()
        .get("referer")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("-")
        .to_string();
    
    let response = next.run(req).await;
    
    let elapsed = start.elapsed();
    let status = response.status();
    let content_length = response.headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("-");
    
    // Format: IP - - [timestamp] "METHOD path HTTP/version" status content_length "referer" "user_agent" duration
    let timestamp = chrono::Utc::now().format("%d/%b/%Y:%H:%M:%S %z");
    info!(
        "{} - - [{}] \"{} {} {:?}\" {} {} \"{}\" \"{}\" {:.3}ms",
        addr.ip(),
        timestamp,
        method,
        uri,
        version,
        status.as_u16(),
        content_length,
        referer,
        user_agent,
        elapsed.as_secs_f64() * 1000.0
    );
    
    response
}

#[derive(Parser)]
#[command(name = "sandbox-service")]
#[command(about = "A secure sandbox service for running TypeScript/Bun/Node.js code")]
struct Args {
    #[arg(short, long, help = "Configuration file path")]
    config: Option<PathBuf>,
    
    #[arg(long, help = "Server host")]
    host: Option<String>,
    
    #[arg(short, long, help = "Server port")]
    port: Option<u16>,
    
    #[arg(short, long, help = "Sandbox backend (docker, nsjail)")]
    backend: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    let mut config = if let Some(config_path) = args.config {
        Config::from_file(&config_path)?
    } else {
        Config::from_env()
    };

    if let Some(host) = args.host {
        config.server.host = host;
    }
    
    if let Some(port) = args.port {
        config.server.port = port;
    }
    
    if let Some(backend) = args.backend {
        config.sandbox.backend = match backend.to_lowercase().as_str() {
            "docker" => sandbox::backend::SandboxBackendType::Docker,
            "nsjail" => sandbox::backend::SandboxBackendType::Nsjail,
            _ => {
                warn!("Unknown backend '{}', using nsjail", backend);
                sandbox::backend::SandboxBackendType::Nsjail
            }
        };
    }

    init_tracing(&config.logging.level)?;

    info!("Starting sandbox service with backend: {:?}", config.sandbox.backend);

    let sandbox_manager = SandboxManager::new(config.sandbox.backend.clone()).await?;
    let app_state = Arc::new(RwLock::new(sandbox_manager));
    
    // Create FaaS state
    let base_url = format!("http://{}:{}", config.server.host, config.server.port);
    let faas_state = FaasState::new(app_state.clone(), base_url);
    
    // Start FaaS cleanup task
    faas_state.faas_manager.start_cleanup_task().await;
    
    // Create proxy state for handling sandbox web services
    let proxy_state = ProxyState::new(8080) // Start port allocation from 8080
        .with_faas_manager(faas_state.faas_manager.clone());

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let api_router = create_router(app_state.clone());
    let faas_router = create_faas_router(faas_state);
    let proxy_router = create_proxy_router(proxy_state);
    let admin_router = create_admin_router(app_state.clone());
    
    let app = Router::new()
        .route("/", axum::routing::get(homepage))
        .merge(api_router)
        .merge(faas_router)
        .merge(proxy_router)
        .merge(admin_router)
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(access_log_middleware))
                .layer(TraceLayer::new_for_http())
                .layer(cors)
        );

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&addr).await?;
    
    info!("Sandbox service listening on {}", addr);
    info!("Health check: http://{}/health", addr);

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal(app_state))
        .await?;

    Ok(())
}

fn init_tracing(level: &str) -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(match level.to_lowercase().as_str() {
            "trace" => tracing::Level::TRACE,
            "debug" => tracing::Level::DEBUG,
            "info" => tracing::Level::INFO,
            "warn" => tracing::Level::WARN,
            "error" => tracing::Level::ERROR,
            _ => tracing::Level::INFO,
        })
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

async fn shutdown_signal(app_state: Arc<RwLock<SandboxManager>>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Received shutdown signal, cleaning up...");
    
    let mut manager = app_state.write().await;
    if let Err(e) = manager.cleanup_all().await {
        warn!("Error during cleanup: {}", e);
    }
    
    info!("Shutdown complete");
}