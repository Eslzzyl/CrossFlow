use image::Luma;
use qrcode::QrCode;
use qrcode::render::svg;

/// 生成 QR 码的 SVG 字符串
pub fn generate_qr_svg(data: &str, size: u32) -> Result<String, Box<dyn std::error::Error>> {
    let code = QrCode::new(data)?;
    
    let svg_string = code.render()
        .min_dimensions(size, size)
        .dark_color(svg::Color("#000000"))
        .light_color(svg::Color("#ffffff"))
        .build();
    
    Ok(svg_string)
}

/// 生成 QR 码的 PNG 字节
pub fn generate_qr_png(data: &str, size: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let code = QrCode::new(data)?;
    
    let image = code.render::<Luma<u8>>()
        .min_dimensions(size, size)
        .build();
    
    let mut png_data = Vec::new();
    {
        image.write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageFormat::Png)?;
    }
    
    Ok(png_data)
}

/// 生成 QR 码的 Base64 字符串（用于直接嵌入 HTML）
pub fn generate_qr_base64(data: &str, size: u32) -> Result<String, Box<dyn std::error::Error>> {
    let png_data = generate_qr_png(data, size)?;
    let base64 = base64::encode(&png_data);
    Ok(format!("data:image/png;base64,{}", base64))
}

// 简单的 base64 编码实现
mod base64 {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    
    pub fn encode(data: &[u8]) -> String {
        let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
        
        for chunk in data.chunks(3) {
            let mut buf = [0u8; 3];
            for (i, &byte) in chunk.iter().enumerate() {
                buf[i] = byte;
            }
            
            let b = ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);
            
            result.push(ALPHABET[((b >> 18) & 0x3F) as usize] as char);
            result.push(ALPHABET[((b >> 12) & 0x3F) as usize] as char);
            
            if chunk.len() > 1 {
                result.push(ALPHABET[((b >> 6) & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }
            
            if chunk.len() > 2 {
                result.push(ALPHABET[(b & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }
        }
        
        result
    }
}
