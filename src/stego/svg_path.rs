/*!
SVG Path Steganography Implementation

This module implements steganography using SVG path animations. The technique works by:
- Encoding data bytes into SVG path coordinates and control points
- Each byte is split into x,y coordinates (using modulo and division)
- Creates quadratic Bezier curves that visually mask the embedded data
- The data is hidden in the precise coordinate values while maintaining valid SVG paths

Key features:
- Maintains visual plausibility through smooth curve paths
- Leverages SVG animation capabilities for data hiding
- Good capacity while preserving visual quality
- Resistant to casual visual inspection

Usage scenarios:
- Hiding data in vector graphics/animations
- Web-based steganography applications
- Cases requiring visual steganography with SVG support
*/

use crate::Result;
use tracing::{debug, info, warn};

/// Convert bytes to SVG path animation
fn byte_to_path(byte: u8, index: usize) -> String {
    let x = (byte % 16) * 10;
    let y = (byte / 16) * 10;

    format!(
        r#"<path id="p{}" d="M {},{} Q{},{} {},{}">
            <animate
                attributeName="d"
                dur="{}.{}s"
                values="M {},{} Q{},{} {},{};
                       M {},{} Q{},{} {},{}"
                repeatCount="indefinite"/>
        </path>"#,
        index,
        x,
        y,
        x + 10,
        y + 10,
        x + 20,
        y,
        byte % 3 + 1,
        byte % 10,
        x,
        y,
        x + 10,
        y + 10,
        x + 20,
        y,
        x + 5,
        y + 5,
        x + 15,
        y + 15,
        x + 25,
        y + 5
    )
}

/// Generate complete SVG document
fn generate_svg_document(paths: &[String]) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Interactive Art</title>
    <style>
        svg {{ width: 100%; height: 100vh; }}
        path {{ stroke: #333; fill: none; stroke-width: 2; }}
    </style>
</head>
<body>
    <svg viewBox="0 0 200 200">
        <defs>
            <filter id="blur">
                <feGaussianBlur stdDeviation="0.5"/>
            </filter>
        </defs>
        {}
    </svg>
</body>
</html>"#,
        paths.join("\n")
    )
}

/// Encode data as SVG path animation
pub fn encode(data: &[u8]) -> Result<Vec<u8>> {
    debug!("Encoding data using SVG path animation steganography");

    if data.is_empty() {
        return Ok(b"<svg viewBox=\"0 0 200 200\"></svg>".to_vec());
    }

    let paths: Vec<String> = data
        .iter()
        .enumerate()
        .map(|(i, &byte)| byte_to_path(byte, i + 1))
        .collect();

    info!("Generated SVG path animation with {} paths", paths.len());
    Ok(generate_svg_document(&paths).into_bytes())
}

/// Decode data from SVG path animation
pub fn decode(data: &[u8]) -> Result<Vec<u8>> {
    debug!("Decoding SVG path animation steganography");

    if data.is_empty() {
        return Ok(Vec::new());
    }

    let content = String::from_utf8_lossy(data);
    let mut result = Vec::new();

    // Use regex to extract path data
    let re = regex::Regex::new(r#"<path[^>]+d="M\s*(\d+),(\d+)"#).unwrap();
    for cap in re.captures_iter(&content) {
        if let (Some(x), Some(y)) = (cap.get(1), cap.get(2)) {
            if let (Ok(x), Ok(y)) = (x.as_str().parse::<u32>(), y.as_str().parse::<u32>()) {
                // Restore byte values from coordinates
                let byte = ((y / 10) * 16 + x / 10) as u8;
                result.push(byte);
                debug!("Decoded coordinates ({},{}) to byte: {}", x, y, byte);
            }
        }
    }

    if !result.is_empty() {
        info!("Successfully decoded {} bytes from SVG paths", result.len());
    } else {
        warn!("No data found in SVG paths");
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_path() {
        let test_data = b"Hello, SVG Path Steganography!";
        let encoded = encode(test_data).unwrap();
        assert!(!encoded.is_empty());
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, test_data);
    }

    #[test]
    fn test_empty_data() {
        let test_data = b"";
        let encoded = encode(test_data).unwrap();
        assert!(!encoded.is_empty());
        let decoded = decode(&encoded).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_large_data() {
        let test_data: Vec<u8> = (0..2000).map(|i| (i % 256) as u8).collect();
        let encoded = encode(&test_data).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert!(!decoded.is_empty());
    }

    #[test]
    fn test_invalid_input() {
        let result = decode(b"").unwrap();
        assert!(result.is_empty());
        let result = decode(b"invalid content").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_path_encoding() {
        let byte = 123;
        let path = byte_to_path(byte, 0);
        assert!(path.contains("path"));
        assert!(path.contains("animate"));
        assert!(path.contains("attributeName=\"d\""));
    }
}
