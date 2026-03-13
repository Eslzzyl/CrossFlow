use crate::models::file::{AppState, ServerInfo};
use crate::services::network::{find_available_port, generate_server_info, generate_server_urls};
use crate::services::qr_service::generate_qr_svg;
use crate::server::run_server;
use std::path::PathBuf;
use tauri::State;

/// 设置共享目录
#[tauri::command]
pub fn set_shared_dir(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let path_buf = PathBuf::from(path);
    if !path_buf.exists() {
        return Err("Path does not exist".to_string());
    }
    if !path_buf.is_dir() {
        return Err("Path is not a directory".to_string());
    }
    
    let mut shared_dir = state.shared_dir.lock().map_err(|e| e.to_string())?;
    *shared_dir = Some(path_buf);
    Ok(())
}

/// 获取当前共享目录
#[tauri::command]
pub fn get_shared_dir(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let shared_dir = state.shared_dir.lock().map_err(|e| e.to_string())?;
    Ok(shared_dir.as_ref().map(|p| p.to_string_lossy().to_string()))
}

/// 启动 HTTP 服务器
#[tauri::command]
pub async fn start_server(
    port: Option<u16>,
    state: State<'_, AppState>
) -> Result<ServerInfo, String> {
    // 检查是否已设置共享目录
    let shared_dir = {
        let shared_dir = state.shared_dir.lock().map_err(|e| e.to_string())?;
        shared_dir.as_ref().ok_or("Shared directory not set")?.clone()
    };
    
    // 检查服务器是否已在运行
    {
        let server_handle = state.server_handle.lock().map_err(|e| e.to_string())?;
        if server_handle.is_some() {
            // 返回已存在的服务器信息
            let server_info = state.server_info.lock().map_err(|e| e.to_string())?;
            return server_info.clone().ok_or("Server info not available".to_string());
        }
    }
    
    // 查找可用端口
    let port = match port {
        Some(p) => p,
        None => find_available_port(8080).await.ok_or("No available port found")?,
    };
    
    // 生成服务器信息
    let server_info = generate_server_info(port).ok_or("Failed to generate server info")?;
    
    // 启动服务器
    let handle = tokio::spawn(async move {
        if let Err(e) = run_server(port, shared_dir).await {
            eprintln!("Server error: {}", e);
        }
    });
    
    // 保存服务器句柄和信息
    {
        let mut server_handle = state.server_handle.lock().map_err(|e| e.to_string())?;
        *server_handle = Some(handle);
    }
    {
        let mut server_info_guard = state.server_info.lock().map_err(|e| e.to_string())?;
        *server_info_guard = Some(server_info.clone());
    }
    
    Ok(server_info)
}

/// 停止 HTTP 服务器
#[tauri::command]
pub fn stop_server(state: State<'_, AppState>) -> Result<(), String> {
    let mut server_handle = state.server_handle.lock().map_err(|e| e.to_string())?;
    if let Some(handle) = server_handle.take() {
        handle.abort();
    }
    
    let mut server_info = state.server_info.lock().map_err(|e| e.to_string())?;
    *server_info = None;
    
    Ok(())
}

/// 获取服务器状态
#[tauri::command]
pub fn get_server_status(state: State<'_, AppState>) -> Result<Option<ServerInfo>, String> {
    let server_info = state.server_info.lock().map_err(|e| e.to_string())?;
    Ok(server_info.clone())
}

/// 生成 QR 码
#[tauri::command]
pub fn generate_qr_code(data: String, size: Option<u32>) -> Result<String, String> {
    let size = size.unwrap_or(256);
    generate_qr_svg(&data, size).map_err(|e| e.to_string())
}

/// 获取所有可用的服务器地址
#[tauri::command]
pub fn get_server_addresses(port: u16) -> Result<Vec<ServerInfo>, String> {
    Ok(generate_server_urls(port))
}

/// 选择目录（调用系统对话框）
#[tauri::command]
pub async fn select_directory(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    
    let result = app.dialog()
        .file()
        .blocking_pick_folder();
    
    Ok(result.map(|p| p.to_string()))
}
