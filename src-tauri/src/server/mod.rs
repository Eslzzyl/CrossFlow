pub mod handlers;
pub mod routes;
pub mod web_ui;

use crate::services::file_service::FileService;
use axum::Router;
use axum::extract::DefaultBodyLimit;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

/// HTTP 服务器
pub struct HttpServer {
    port: u16,
    shared_dir: PathBuf,
    password: Option<String>,
}

impl HttpServer {
    pub fn new(port: u16, shared_dir: PathBuf, password: Option<String>) -> Self {
        Self { port, shared_dir, password }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file_service = Arc::new(FileService::new(self.shared_dir.clone()));

        // CORS 配置 - 允许局域网访问
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        // 请求体大小限制：100GB
        let body_limit = RequestBodyLimitLayer::new(100 * 1024 * 1024 * 1024);

        let app = Router::new()
            .merge(routes::create_routes(file_service, self.password.clone()))
            .layer(cors)
            .layer(DefaultBodyLimit::disable())
            .layer(body_limit)
            .layer(TraceLayer::new_for_http());

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        info!("Starting HTTP server on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

/// 启动服务器（用于在单独的任务中运行）
pub async fn run_server(port: u16, shared_dir: PathBuf, password: Option<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let file_service = Arc::new(FileService::new(shared_dir));

    // CORS 配置
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 请求体大小限制：100GB
    let body_limit = RequestBodyLimitLayer::new(100 * 1024 * 1024 * 1024);

    let app = Router::new()
        .merge(routes::create_routes(file_service, password))
        .layer(cors)
        .layer(DefaultBodyLimit::disable())
        .layer(body_limit)
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Starting HTTP server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

    axum::serve(listener, app).await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

    Ok(())
}
