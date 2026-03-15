use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// 设备信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct Device {
    pub id: String,
    pub ip: String,
    pub user_agent: String,
    pub device_name: String,
    pub device_type: DeviceType,
    pub first_seen: u64,
    pub last_seen: u64,
}

/// 设备类型
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    Mobile,
    Tablet,
    Desktop,
    Unknown,
}

/// 设备追踪器
pub struct DeviceTracker {
    devices: Arc<RwLock<HashMap<String, Device>>>,
    timeout: Duration,
}

impl DeviceTracker {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
            timeout: Duration::from_secs(120), // 2分钟超时
        }
    }

    /// 记录设备访问
    pub async fn record_visit(&self, ip: IpAddr, user_agent: &str) -> Device {
        let device_id = format!("{}-{}", ip, self.hash_user_agent(user_agent));
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let device_type = self.detect_device_type(user_agent);
        let device_name = self.extract_device_name(user_agent);

        let mut devices = self.devices.write().await;

        let device = if let Some(existing) = devices.get(&device_id) {
            Device {
                id: device_id.clone(),
                ip: ip.to_string(),
                user_agent: user_agent.to_string(),
                device_name: existing.device_name.clone(),
                device_type: existing.device_type.clone(),
                first_seen: existing.first_seen,
                last_seen: timestamp,
            }
        } else {
            Device {
                id: device_id.clone(),
                ip: ip.to_string(),
                user_agent: user_agent.to_string(),
                device_name,
                device_type,
                first_seen: timestamp,
                last_seen: timestamp,
            }
        };

        devices.insert(device_id, device.clone());
        device
    }

    /// 获取所有活跃设备
    pub async fn get_active_devices(&self) -> Vec<Device> {
        // 先清理超时设备
        self.cleanup_devices().await;

        // 返回剩余的所有设备
        let devices = self.devices.read().await;
        devices.values().cloned().collect()
    }

    /// 清理超时设备
    pub async fn cleanup_devices(&self) {
        let mut devices = self.devices.write().await;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        devices.retain(|_, device| now - device.last_seen < self.timeout.as_secs());
    }

    /// 检测设备类型
    fn detect_device_type(&self, user_agent: &str) -> DeviceType {
        let ua = user_agent.to_lowercase();
        if ua.contains("mobile")
            || ua.contains("android")
            || ua.contains("iphone")
            || ua.contains("ipod")
        {
            DeviceType::Mobile
        } else if ua.contains("tablet") || ua.contains("ipad") {
            DeviceType::Tablet
        } else if ua.contains("windows")
            || ua.contains("macintosh")
            || ua.contains("linux")
        {
            DeviceType::Desktop
        } else {
            DeviceType::Unknown
        }
    }

    /// 提取设备名称
    fn extract_device_name(&self, user_agent: &str) -> String {
        let ua = user_agent.to_lowercase();

        if ua.contains("iphone") {
            "iPhone".to_string()
        } else if ua.contains("ipad") {
            "iPad".to_string()
        } else if ua.contains("android") {
            if ua.contains("mobile") {
                "Android Phone".to_string()
            } else {
                "Android Tablet".to_string()
            }
        } else if ua.contains("windows") {
            "Windows PC".to_string()
        } else if ua.contains("macintosh") || ua.contains("mac os") {
            "Mac".to_string()
        } else if ua.contains("linux") {
            "Linux".to_string()
        } else {
            "Unknown Device".to_string()
        }
    }

    /// 生成 User-Agent 的简单哈希
    fn hash_user_agent(&self, user_agent: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        user_agent.hash(&mut hasher);
        format!("{:x}", hasher.finish())[..8].to_string()
    }
}

impl Default for DeviceTracker {
    fn default() -> Self {
        Self::new()
    }
}
