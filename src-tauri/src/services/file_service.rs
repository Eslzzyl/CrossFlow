use crate::models::file::{DirectoryListing, FileInfo};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileServiceError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Path not allowed: {0}")]
    PathNotAllowed(String),
    #[error("Not a directory: {0}")]
    NotADirectory(String),
}

pub struct FileService {
    base_dir: PathBuf,
}

impl FileService {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// 规范化路径，确保在 base_dir 内
    fn sanitize_path(&self, requested_path: &str) -> Result<PathBuf, FileServiceError> {
        let decoded = percent_encoding::percent_decode_str(requested_path)
            .decode_utf8_lossy()
            .to_string();

        let requested = PathBuf::from(&decoded);
        
        // 如果是绝对路径，检查是否在 base_dir 内
        let full_path = if requested.is_absolute() {
            requested
        } else {
            self.base_dir.join(&requested)
        };

        // 规范化路径
        let canonical_base = self.base_dir.canonicalize()?;
        let canonical_requested = full_path.canonicalize().or_else(|_| {
            // 如果路径不存在，尝试规范化父目录
            if let Some(parent) = full_path.parent() {
                let canonical_parent = parent.canonicalize()?;
                Ok(canonical_parent.join(full_path.file_name().unwrap_or_default()))
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Invalid path",
                ))
            }
        })?;

        // 安全检查：确保请求的路径在 base_dir 内
        if !canonical_requested.starts_with(&canonical_base) {
            return Err(FileServiceError::PathNotAllowed(
                canonical_requested.to_string_lossy().to_string(),
            ));
        }

        Ok(canonical_requested)
    }

    /// 列出目录内容
    pub fn list_directory(&self, path: &str) -> Result<DirectoryListing, FileServiceError> {
        let full_path = self.sanitize_path(path)?;

        if !full_path.is_dir() {
            return Err(FileServiceError::NotADirectory(
                full_path.to_string_lossy().to_string(),
            ));
        }

        let mut files = Vec::new();
        let mut entries: Vec<_> = std::fs::read_dir(&full_path)?.collect();
        
        // 排序：目录在前，文件在后，按名称排序
        entries.sort_by_key(|e| {
            let path = e.as_ref().unwrap().path();
            let is_dir = path.is_dir();
            let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            (!is_dir, name)
        });

        for entry in entries {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let name = entry.file_name().to_string_lossy().to_string();
            
            // 计算相对路径
            let relative_path = if path.is_empty() || path == "/" {
                name.clone()
            } else {
                format!("{}/{}", path.trim_end_matches('/'), name)
            };

            let modified = metadata
                .modified()
                .ok()
                .and_then(|t| {
                    t.duration_since(std::time::UNIX_EPOCH)
                        .ok()
                        .map(|d| d.as_secs())
                })
                .map(|ts| {
                    chrono::DateTime::from_timestamp(ts as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_default()
                });

            files.push(FileInfo {
                name,
                path: relative_path,
                is_dir: metadata.is_dir(),
                size: if metadata.is_file() { Some(metadata.len()) } else { None },
                modified,
            });
        }

        // 计算父目录路径
        let parent_path = if full_path == self.base_dir.canonicalize()? {
            None
        } else {
            let parent = PathBuf::from(path).parent()
                .map(|p| p.to_string_lossy().to_string())
                .filter(|p| !p.is_empty());
            parent.or(Some("".to_string()))
        };

        Ok(DirectoryListing {
            current_path: path.to_string(),
            parent_path,
            files,
        })
    }

    /// 获取文件路径用于下载
    pub fn get_file_path(&self, path: &str) -> Result<PathBuf, FileServiceError> {
        let full_path = self.sanitize_path(path)?;
        
        if !full_path.is_file() {
            return Err(FileServiceError::NotADirectory(
                "Not a file".to_string(),
            ));
        }

        Ok(full_path)
    }

    /// 获取基础目录
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// 保存上传的文件
    pub fn save_file(&self, relative_path: &str, data: &[u8]) -> Result<PathBuf, FileServiceError> {
        let full_path = self.sanitize_path(relative_path)?;

        // 确保父目录存在
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&full_path, data)?;

        Ok(full_path)
    }

    /// 获取保存路径（不实际保存文件）
    pub fn get_save_path(&self, relative_path: &str) -> Result<PathBuf, FileServiceError> {
        self.sanitize_path(relative_path)
    }
}

/// 格式化文件大小
pub fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}
