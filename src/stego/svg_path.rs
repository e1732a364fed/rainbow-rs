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
use rand::Rng;
use tracing::{debug, info, trace, warn};

use crate::stego::{Encoder, Random};

#[derive(Debug, Clone)]

pub struct SvgPathEncoder {
    viewbox_size: (u32, u32),
}

impl Random for SvgPathEncoder {
    fn random() -> Self {
        let mut rng = rand::thread_rng();
        let width = rng.gen_range(400..1200);
        let height = (width as f32 * rng.gen_range(0.5..1.5)) as u32;
        Self {
            viewbox_size: (width, height),
        }
    }
}

impl Default for SvgPathEncoder {
    fn default() -> Self {
        Self {
            viewbox_size: (800, 600),
        }
    }
}

impl Encoder for SvgPathEncoder {
    fn name(&self) -> &'static str {
        "svg_path"
    }

    fn encode(&self, data: &[u8]) -> Result<Vec<u8>> {
        encode(data, self.viewbox_size)
    }

    fn decode(&self, content: &[u8]) -> Result<Vec<u8>> {
        decode(content)
    }

    fn get_mime_type(&self) -> &'static str {
        "image/svg+xml"
    }
}

/// Convert bytes to SVG path animation
fn byte_to_path(byte: u8, index: usize) -> String {
    let x = (byte % 16) * 10;
    let y = (byte / 16) * 10;

    format!(
        r#"<path id="p{}" d="M {},{} Q{},{} {},{}">
            <animate
                attributeName="d"
                dur="{}.{}s"
                values="M {},{} Q{},{} {},{}
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

/*
<!DOCTYPE html>
<html>
<head>
    <title>Interactive Art</title>
    <style>
        svg {{ width: 100%; height: 100vh; }}
        path {{ stroke: #333; fill: none; stroke-width: 2; }}
    </style>
</head>
<body>
</body>
</html>
*/

/// Generate complete SVG document
fn generate_svg_document(paths: &[String], viewbox_size: (u32, u32)) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" version="1.1" viewBox="0 0 {} {}">
        <defs>
            <filter id="blur">
                <feGaussianBlur stdDeviation="0.5"/>
            </filter>
        </defs>
        {}
    </svg>"#,
        viewbox_size.0,
        viewbox_size.1,
        paths.join("\n")
    )
}

/// Encode data as SVG path animation
pub fn encode(data: &[u8], viewbox_size: (u32, u32)) -> Result<Vec<u8>> {
    debug!("Encoding data using SVG path animation steganography");

    if data.is_empty() {
        return Ok(format!(
            "<svg viewBox=\"0 0 {} {}\"></svg>",
            viewbox_size.0, viewbox_size.1
        )
        .into_bytes());
    }

    let paths: Vec<String> = data
        .iter()
        .enumerate()
        .map(|(i, &byte)| byte_to_path(byte, i + 1))
        .collect();

    debug!("Generated SVG path animation with {} paths", paths.len());
    Ok(generate_svg_document(&paths, viewbox_size).into_bytes())
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
                trace!("Decoded coordinates ({},{}) to byte: {}", x, y, byte);
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
        let encoded = encode(test_data, (800, 600)).unwrap();
        assert!(!encoded.is_empty());
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, test_data);
    }

    #[test]
    fn test_empty_data() {
        let test_data = b"";
        let encoded = encode(test_data, (800, 600)).unwrap();
        assert!(!encoded.is_empty());
        let decoded = decode(&encoded).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_large_data() {
        let test_data: Vec<u8> = (0..2000).map(|i| (i % 256) as u8).collect();
        let encoded = encode(&test_data, (800, 600)).unwrap();
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
