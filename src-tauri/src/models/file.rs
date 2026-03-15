use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use crate::services::device_tracker::DeviceTracker;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub modified: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryListing {
    pub current_path: String,
    pub parent_path: Option<String>,
    pub files: Vec<FileInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub address: String,
    pub port: u16,
    pub url: String,
}

pub struct AppState {
    pub shared_dir: std::sync::Mutex<Option<PathBuf>>,
    pub server_handle: std::sync::Mutex<Option<tokio::task::JoinHandle<()>>>,
    pub server_info: std::sync::Mutex<Option<ServerInfo>>,
    pub server_password: std::sync::Mutex<Option<String>>,
    pub device_tracker: Arc<DeviceTracker>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            shared_dir: std::sync::Mutex::new(None),
            server_handle: std::sync::Mutex::new(None),
            server_info: std::sync::Mutex::new(None),
            server_password: std::sync::Mutex::new(None),
            device_tracker: Arc::new(DeviceTracker::new()),
        }
    }
}
