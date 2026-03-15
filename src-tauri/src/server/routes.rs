use crate::server::handlers::{self, AuthConfig};
use crate::services::device_tracker::DeviceTracker;
use crate::services::file_service::FileService;
use axum::{
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, post},
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;

/// 认证中间件
async fn auth_middleware(
    State(auth_config): State<Arc<AuthConfig>>,
    request: Request,
    next: Next,
) -> Response {
    // 如果未启用密码保护，直接通过
    if !auth_config.is_enabled() {
        return next.run(request).await;
    }

    // 检查认证头
    let token = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if auth_config.verify(token) {
        next.run(request).await
    } else {
        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(axum::body::Body::from(r#"{"success":false,"error":"Unauthorized"}"#))
            .unwrap()
    }
}

/// 设备追踪中间件
async fn device_tracking_middleware(
    State(device_tracker): State<Arc<DeviceTracker>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    // 获取 User-Agent
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("Unknown");

    // 记录设备访问
    let _ = device_tracker.record_visit(addr.ip(), user_agent).await;

    next.run(request).await
}

/// 创建路由
pub fn create_routes(
    file_service: Arc<FileService>,
    device_tracker: Arc<DeviceTracker>,
    password: Option<String>,
) -> Router {
    let auth_config = Arc::new(AuthConfig::new(password));

    // 需要保护的路由
    let protected_routes = Router::new()
        .route("/api/files", get(handlers::list_files_handler))
        .route("/api/download/{*path}", get(handlers::download_file_handler))
        .route("/api/preview/{*path}", get(handlers::preview_file_handler))
        .route("/api/upload", post(handlers::upload_file_handler))
        .route("/api/delete", delete(handlers::delete_file_handler))
        .route_layer(middleware::from_fn_with_state(auth_config.clone(), auth_middleware))
        .with_state(file_service.clone());

    // 公开路由
    let public_routes = Router::new()
        .route("/", get(handlers::web_ui_handler))
        .route("/api/health", get(handlers::health_handler))
        .route("/api/auth/check", get(handlers::auth_check_handler))
        .route("/api/auth/login", post(handlers::auth_login_handler))
        .with_state(auth_config);

    Router::new()
        .merge(protected_routes)
        .merge(public_routes)
        .layer(middleware::from_fn_with_state(device_tracker, device_tracking_middleware))
}
