use crate::models::file::ServerInfo;
use local_ip_address::local_ip;
use std::net::UdpSocket;

/// 获取本机局域网 IP 地址
pub fn get_local_ip() -> Option<String> {
    local_ip().ok().map(|ip| ip.to_string())
}

/// 通过 UDP 连接获取本地 IP
fn get_local_ip_via_udp() -> Option<String> {
    // 连接到一个公共 DNS 服务器，获取本地出口 IP
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let local_addr = socket.local_addr().ok()?;
    Some(local_addr.ip().to_string())
}

/// 查找可用端口
pub async fn find_available_port(start_port: u16) -> Option<u16> {
    for port in start_port..=65535 {
        if is_port_available(port).await {
            return Some(port);
        }
    }
    None
}

/// 检查端口是否可用
async fn is_port_available(port: u16) -> bool {
    tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .is_ok()
}

/// 生成服务器信息
pub fn generate_server_info(port: u16) -> Option<ServerInfo> {
    // 优先使用 local_ip_address crate
    let ip = get_local_ip()
        .or_else(get_local_ip_via_udp)
        .or_else(|| Some("127.0.0.1".to_string()))?;
    
    let address = format!("{}:{}", ip, port);
    let url = format!("http://{}", address);
    
    Some(ServerInfo {
        address,
        port,
        url,
    })
}

/// 获取所有本机 IP 地址
pub fn get_all_local_ips() -> Vec<String> {
    let mut ips = Vec::new();
    
    // 尝试获取本地 IP
    if let Some(ip) = get_local_ip() {
        if !ip.starts_with("127.") {
            ips.push(ip);
        }
    }
    
    // 尝试通过 UDP 获取
    if let Some(ip) = get_local_ip_via_udp() {
        if !ip.starts_with("127.") && !ips.contains(&ip) {
            ips.push(ip);
        }
    }
    
    // 如果上面都失败了，使用本地 IP
    if ips.is_empty() {
        ips.push("127.0.0.1".to_string());
    }
    
    ips
}

/// 生成服务器 URL 列表
pub fn generate_server_urls(port: u16) -> Vec<ServerInfo> {
    let ips = get_all_local_ips();
    let mut servers = Vec::new();
    
    for ip in ips {
        // 过滤掉回环地址（保留 127.0.0.1 作为备选）
        if ip.starts_with("127.") && servers.len() > 0 {
            continue;
        }
        
        let address = format!("{}:{}", ip, port);
        let url = format!("http://{}", address);
        
        servers.push(ServerInfo {
            address,
            port,
            url,
        });
    }
    
    servers
}
