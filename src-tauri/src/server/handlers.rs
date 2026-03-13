use crate::services::file_service::FileService;
use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    Json,
};
use serde::Deserialize;
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

/// Web UI 页面
pub async fn web_ui_handler() -> impl IntoResponse {
    Html(include_str!("web_ui.html"))
}
