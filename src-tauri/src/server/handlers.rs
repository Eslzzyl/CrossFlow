use crate::services::file_service::FileService;
use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct ListFilesQuery {
    path: Option<String>,
}

/// 健康检查
pub async fn health_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().timestamp()
    }))
}

/// 列出文件
pub async fn list_files_handler(
    State(file_service): State<Arc<FileService>>,
    Query(query): Query<ListFilesQuery>,
) -> impl IntoResponse {
    let path = query.path.unwrap_or_default();
    
    match file_service.list_directory(&path) {
        Ok(listing) => Json(serde_json::json!({
            "success": true,
            "data": listing
        })).into_response(),
        Err(e) => {
            let status = match e {
                crate::services::file_service::FileServiceError::PathNotAllowed(_) => StatusCode::FORBIDDEN,
                crate::services::file_service::FileServiceError::NotADirectory(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            
            (status, Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))).into_response()
        }
    }
}

/// 下载文件
pub async fn download_file_handler(
    State(file_service): State<Arc<FileService>>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    match file_service.get_file_path(&path) {
        Ok(file_path) => {
            let file_name = file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            
            let mime_type = mime_guess::from_path(&file_path)
                .first_or_octet_stream()
                .to_string();

            match tokio::fs::read(&file_path).await {
                Ok(content) => {
                    Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, mime_type)
                        .header(
                            header::CONTENT_DISPOSITION,
                            format!("attachment; filename=\"{}\"", file_name),
                        )
                        .body(Body::from(content))
                        .unwrap()
                }
                Err(e) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read file: {}", e))
                        .into_response()
                }
            }
        }
        Err(e) => {
            let status = match e {
                crate::services::file_service::FileServiceError::PathNotAllowed(_) => StatusCode::FORBIDDEN,
                _ => StatusCode::NOT_FOUND,
            };
            
            (status, e.to_string()).into_response()
        }
    }
}

/// 上传文件
pub async fn upload_file_handler(
    State(file_service): State<Arc<FileService>>,
    Query(query): Query<ListFilesQuery>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut uploaded_files = Vec::new();
    let target_dir = query.path.unwrap_or_default();

    loop {
        let mut field = match multipart.next_field().await {
            Ok(Some(field)) => field,
            Ok(None) => break,
            Err(e) => {
                tracing::error!("Multipart error: {}", e);
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "success": false,
                        "error": format!("Multipart error: {}", e)
                    }))
                ).into_response();
            }
        };

        let file_name = field.file_name().unwrap_or("unnamed").to_string();

        // 确定保存路径
        let save_path = if target_dir.is_empty() {
            file_name.clone()
        } else {
            format!("{}/{}", target_dir.trim_end_matches('/'), file_name)
        };

        // 获取完整保存路径
        let full_path = match file_service.get_save_path(&save_path) {
            Ok(path) => path,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "success": false,
                        "error": format!("Invalid save path for {}: {}", file_name, e)
                    }))
                ).into_response();
            }
        };

        // 确保父目录存在
        if let Some(parent) = full_path.parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "success": false,
                        "error": format!("Failed to create directory: {}", e)
                    }))
                ).into_response();
            }
        }

        // 创建文件并流式写入
        let mut file = match tokio::fs::File::create(&full_path).await {
            Ok(f) => f,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "success": false,
                        "error": format!("Failed to create file {}: {}", file_name, e)
                    }))
                ).into_response();
            }
        };

        // 流式读取并写入
        let mut total_size: u64 = 0;
        loop {
            let chunk = match field.chunk().await {
                Ok(Some(chunk)) => chunk,
                Ok(None) => break,
                Err(e) => {
                    tracing::error!("Chunk error for {}: {}", file_name, e);
                    let _ = tokio::fs::remove_file(&full_path).await;
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({
                            "success": false,
                            "error": format!("Chunk error for {}: {}", file_name, e)
                        }))
                    ).into_response();
                }
            };
            
            total_size += chunk.len() as u64;
            if let Err(e) = tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await {
                // 清理失败的文件
                let _ = tokio::fs::remove_file(&full_path).await;
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "success": false,
                        "error": format!("Failed to write file {}: {}", file_name, e)
                    }))
                ).into_response();
            }
        }

        // 刷新文件到磁盘
        if let Err(e) = tokio::io::AsyncWriteExt::flush(&mut file).await {
            let _ = tokio::fs::remove_file(&full_path).await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to flush file {}: {}", file_name, e)
                }))
            ).into_response();
        }

        uploaded_files.push(serde_json::json!({
            "file_name": file_name,
            "path": save_path,
            "size": total_size,
            "saved_to": full_path.to_string_lossy().to_string()
        }));
    }

    Json(serde_json::json!({
        "success": true,
        "files": uploaded_files
    })).into_response()
}

/// 删除文件或目录
pub async fn delete_file_handler(
    State(file_service): State<Arc<FileService>>,
    Query(query): Query<ListFilesQuery>,
) -> impl IntoResponse {
    let path = query.path.unwrap_or_default();

    if path.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": "Path is required"
            }))
        ).into_response();
    }

    match file_service.delete_path(&path) {
        Ok(_) => Json(serde_json::json!({
            "success": true,
            "message": "Deleted successfully"
        })).into_response(),
        Err(e) => {
            let status = match e {
                crate::services::file_service::FileServiceError::PathNotAllowed(_) => StatusCode::FORBIDDEN,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            (status, Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))).into_response()
        }
    }
}

// ========== 认证相关 ==========

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub password: Option<String>,
}

impl AuthConfig {
    pub fn new(password: Option<String>) -> Self {
        Self { password }
    }

    pub fn is_enabled(&self) -> bool {
        self.password.is_some()
    }

    pub fn verify(&self, token: &str) -> bool {
        if let Some(ref pwd) = self.password {
            // 简单的 token 验证：token = "Bearer " + password 的 base64
            use base64::{Engine as _, engine::general_purpose};
            let expected = format!("Bearer {}", general_purpose::STANDARD.encode(pwd));
            token == expected
        } else {
            true
        }
    }

    pub fn generate_token(&self) -> Option<String> {
        use base64::{Engine as _, engine::general_purpose};
        self.password.as_ref().map(|pwd| format!("Bearer {}", general_purpose::STANDARD.encode(pwd)))
    }
}

#[derive(Deserialize)]
pub struct LoginRequest {
    password: String,
}

#[derive(Serialize)]
pub struct AuthCheckResponse {
    require_auth: bool,
    authenticated: bool,
}

/// 检查认证状态
pub async fn auth_check_handler(
    State(auth_config): State<Arc<AuthConfig>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let authenticated = !auth_config.is_enabled() || auth_config.verify(token);

    Json(AuthCheckResponse {
        require_auth: auth_config.is_enabled(),
        authenticated,
    })
}

/// 登录验证
pub async fn auth_login_handler(
    State(auth_config): State<Arc<AuthConfig>>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    if let Some(ref expected_pwd) = auth_config.password {
        if req.password == *expected_pwd {
            if let Some(token) = auth_config.generate_token() {
                return Json(serde_json::json!({
                    "success": true,
                    "token": token
                })).into_response();
            }
        }

        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "success": false,
                "error": "Invalid password"
            }))
        ).into_response();
    }

    Json(serde_json::json!({
        "success": true,
        "message": "No password required"
    })).into_response()
}

/// 预览文件
/// GET /api/preview/{path}
/// 支持图片、视频（含Range请求）、文本文件的预览
pub async fn preview_file_handler(
    State(file_service): State<Arc<FileService>>,
    Path(path): Path<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    const MAX_TEXT_SIZE: u64 = 10 * 1024 * 1024; // 10MB

    match file_service.get_file_path(&path) {
        Ok(file_path) => {
            let mime_type = mime_guess::from_path(&file_path)
                .first_or_octet_stream()
                .to_string();

            // 判断文件类型
            let is_image = mime_type.starts_with("image/");
            let is_video = mime_type.starts_with("video/");
            let is_text = mime_type.starts_with("text/")
                || mime_type == "application/json"
                || mime_type == "application/xml"
                || mime_type == "application/javascript"
                || mime_type == "application/typescript";

            if is_text {
                // 文本文件：读取内容返回 JSON
                match tokio::fs::metadata(&file_path).await {
                    Ok(metadata) => {
                        if metadata.len() > MAX_TEXT_SIZE {
                            return (
                                StatusCode::PAYLOAD_TOO_LARGE,
                                Json(serde_json::json!({
                                    "success": false,
                                    "error": "File too large for preview (max 10MB)"
                                }))
                            ).into_response();
                        }
                    }
                    Err(e) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({
                                "success": false,
                                "error": format!("Failed to read file metadata: {}", e)
                            }))
                        ).into_response();
                    }
                }

                match tokio::fs::read_to_string(&file_path).await {
                    Ok(content) => {
                        Json(serde_json::json!({
                            "success": true,
                            "content": content,
                            "mime_type": mime_type
                        })).into_response()
                    }
                    Err(e) => {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({
                                "success": false,
                                "error": format!("Failed to read file: {}", e)
                            }))
                        ).into_response()
                    }
                }
            } else if is_image || is_video {
                // 图片/视频文件：支持 Range 请求
                match tokio::fs::metadata(&file_path).await {
                    Ok(metadata) => {
                        let file_size = metadata.len();

                        // 检查是否有 Range 请求头
                        let range_header = headers
                            .get(header::RANGE)
                            .and_then(|v| v.to_str().ok());

                        if let Some(range) = range_header {
                            // 解析 Range 头 (格式: bytes=start-end)
                            if let Some(range_val) = range.strip_prefix("bytes=") {
                                let parts: Vec<&str> = range_val.split('-').collect();
                                if parts.len() == 2 {
                                    let start = parts[0].parse::<u64>().unwrap_or(0);
                                    let end = if parts[1].is_empty() {
                                        file_size - 1
                                    } else {
                                        parts[1].parse::<u64>().unwrap_or(file_size - 1)
                                    };

                                    if start > end || start >= file_size {
                                        return (
                                            StatusCode::RANGE_NOT_SATISFIABLE,
                                            format!("Range not satisfiable")
                                        ).into_response();
                                    }

                                    let actual_end = end.min(file_size - 1);
                                    let content_length = actual_end - start + 1;

                                    // 读取指定范围的内容
                                    match read_file_range(&file_path, start, content_length).await {
                                        Ok(content) => {
                                            Response::builder()
                                                .status(StatusCode::PARTIAL_CONTENT)
                                                .header(header::CONTENT_TYPE, mime_type)
                                                .header(
                                                    header::CONTENT_RANGE,
                                                    format!("bytes {}-{}/{}", start, actual_end, file_size),
                                                )
                                                .header(header::CONTENT_LENGTH, content_length)
                                                .header(header::ACCEPT_RANGES, "bytes")
                                                .body(Body::from(content))
                                                .unwrap()
                                        }
                                        Err(e) => {
                                            (
                                                StatusCode::INTERNAL_SERVER_ERROR,
                                                format!("Failed to read file range: {}", e)
                                            ).into_response()
                                        }
                                    }
                                } else {
                                    // 返回完整文件
                                    match tokio::fs::read(&file_path).await {
                                        Ok(content) => {
                                            Response::builder()
                                                .status(StatusCode::OK)
                                                .header(header::CONTENT_TYPE, mime_type)
                                                .header(header::CONTENT_LENGTH, file_size)
                                                .header(header::ACCEPT_RANGES, "bytes")
                                                .body(Body::from(content))
                                                .unwrap()
                                        }
                                        Err(e) => {
                                            (
                                                StatusCode::INTERNAL_SERVER_ERROR,
                                                format!("Failed to read file: {}", e)
                                            ).into_response()
                                        }
                                    }
                                }
                            } else {
                                // 返回完整文件
                                match tokio::fs::read(&file_path).await {
                                    Ok(content) => {
                                        Response::builder()
                                            .status(StatusCode::OK)
                                            .header(header::CONTENT_TYPE, mime_type)
                                            .header(header::CONTENT_LENGTH, file_size)
                                            .header(header::ACCEPT_RANGES, "bytes")
                                            .body(Body::from(content))
                                            .unwrap()
                                    }
                                    Err(e) => {
                                        (
                                            StatusCode::INTERNAL_SERVER_ERROR,
                                            format!("Failed to read file: {}", e)
                                        ).into_response()
                                    }
                                }
                            }
                        } else {
                            // 无 Range 请求，返回完整文件
                            match tokio::fs::read(&file_path).await {
                                Ok(content) => {
                                    Response::builder()
                                        .status(StatusCode::OK)
                                        .header(header::CONTENT_TYPE, mime_type)
                                        .header(header::CONTENT_LENGTH, file_size)
                                        .header(header::ACCEPT_RANGES, "bytes")
                                        .body(Body::from(content))
                                        .unwrap()
                                }
                                Err(e) => {
                                    (
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                        format!("Failed to read file: {}", e)
                                    ).into_response()
                                }
                            }
                        }
                    }
                    Err(e) => {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Failed to read file metadata: {}", e)
                        ).into_response()
                    }
                }
            } else {
                // 不支持的文件类型
                (
                    StatusCode::UNSUPPORTED_MEDIA_TYPE,
                    Json(serde_json::json!({
                        "success": false,
                        "error": "Unsupported file type for preview"
                    }))
                ).into_response()
            }
        }
        Err(e) => {
            let status = match e {
                crate::services::file_service::FileServiceError::PathNotAllowed(_) => StatusCode::FORBIDDEN,
                _ => StatusCode::NOT_FOUND,
            };

            (status, Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))).into_response()
        }
    }
}

/// 读取文件的指定范围
async fn read_file_range(
    path: &std::path::Path,
    start: u64,
    length: u64,
) -> Result<Vec<u8>, std::io::Error> {
    use tokio::io::AsyncReadExt;
    use tokio::io::AsyncSeekExt;

    let mut file = tokio::fs::File::open(path).await?;
    file.seek(std::io::SeekFrom::Start(start)).await?;

    let mut buffer = vec![0u8; length as usize];
    let mut total_read = 0;

    while total_read < length as usize {
        match file.read(&mut buffer[total_read..]).await? {
            0 => break, // EOF
            n => total_read += n,
        }
    }

    buffer.truncate(total_read);
    Ok(buffer)
}

/// Web UI 页面
pub async fn web_ui_handler() -> impl IntoResponse {
    Html(include_str!("web_ui.html"))
}
