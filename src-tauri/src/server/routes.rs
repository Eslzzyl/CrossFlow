use crate::server::handlers;
use crate::services::file_service::FileService;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

/// 创建路由
pub fn create_routes(file_service: Arc<FileService>) -> Router {
    Router::new()
        // Web UI
        .route("/", get(handlers::web_ui_handler))
        // API 路由
        .route("/api/files", get(handlers::list_files_handler))
        .route("/api/download/{*path}", get(handlers::download_file_handler))
        .route("/api/upload", post(handlers::upload_file_handler))
        .route("/api/health", get(handlers::health_handler))
        .with_state(file_service)
}
