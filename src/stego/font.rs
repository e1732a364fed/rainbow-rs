/*! Font-based Steganography Implementation

This module implements steganography using font variation settings in text.
The technique works by encoding data into subtle variations of font properties:

- Font weight (thickness): Range 100-1600
- Font width: Range 0-90
- Font slant/italic angle: Range 0-15 degrees

Each byte is split and encoded across these three properties in a way that appears
natural while maintaining the text's readability. This method is particularly
suitable for web content and rich text documents where font variations are common.

Key features:
- High visual imperceptibility
- Maintains text readability
- Compatible with most modern fonts
- Reversible encoding/decoding
*/

use crate::Result;
use regex::Regex;
use tracing::{debug, info, warn};

/// Convert bytes to font variation settings
fn byte_to_font_variation(byte: u8, index: usize) -> String {
    let weight = (byte / 16) as u32 * 100 + 100; // Weight range 100-1600
    let width = (byte % 16) as u32 * 6; // Width range 0-90
    let slant = (byte % 4) as u32 * 5; // Slant angles: 0, 5, 10, 15 degrees

    format!(
        r#".v{} {{
            font-variation-settings: 'wght' {}, 'wdth' {}, 'slnt' {};
            font-family: 'Variable';
        }}"#,
        index, weight, width, slant
    )
}

/// Generate complete HTML document
fn generate_html_document(variations: &[String], chars: &[String]) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Typography Showcase</title>
    <style>
        @font-face {{
            font-family: 'Variable';
            src: url('data:font/woff2;base64,d09GMgABAAA...') format('woff2');
            font-weight: 100 900;
            font-stretch: 25% 151%;
            font-style: oblique 0deg 15deg;
        }}
        body {{
            font-family: 'Variable', sans-serif;
            line-height: 1.5;
        }}
        span {{
            display: inline-block;
            margin: 0.1em;
        }}
        {}
    </style>
</head>
<body>
    <div class="content">
        <h1>Typography Examples</h1>
        {}
        <p>Exploring variable fonts in modern web design.</p>
    </div>
</body>
</html>"#,
        variations.join("\n        "),
        chars.join("\n        ")
    )
}

/// Encode data as font variations
pub fn encode(data: &[u8]) -> Result<Vec<u8>> {
    debug!("Encoding data using font variation steganography");

    if data.is_empty() {
        return Ok(b"<!DOCTYPE html><html><head></head><body></body></html>".to_vec());
    }

    let mut variations = Vec::new();
    let mut chars = Vec::new();

    for (i, &byte) in data.iter().enumerate() {
        // Create font variation style
        variations.push(byte_to_font_variation(byte, i + 1));

        // Create character element with class
        chars.push(format!("<span class=\"v{}\">O</span>", i + 1));
    }

    info!(
        "Generated font variation steganography with {} characters",
        data.len()
    );
    Ok(generate_html_document(&variations, &chars).into_bytes())
}

/// Decode data from font variations
pub fn decode(data: &[u8]) -> Result<Vec<u8>> {
    debug!("Decoding font variation steganography");

    if data.is_empty() {
        return Ok(Vec::new());
    }

    let content = String::from_utf8_lossy(data);
    let mut result = Vec::new();

    // Extract all font variation settings
    let re = Regex::new(
        r"font-variation-settings:\s*'wght'\s*(\d+),\s*'wdth'\s*(\d+),\s*'slnt'\s*(\d+)",
    )
    .unwrap();
    for cap in re.captures_iter(&content) {
        if let (Some(weight), Some(width), Some(slant)) = (
            cap[1].parse::<u32>().ok(),
            cap[2].parse::<u32>().ok(),
            cap[3].parse::<u32>().ok(),
        ) {
            // Restore byte value from font variation parameters
            let byte_value = (((weight - 100) / 100) << 4 | (width / 6)) as u8;
            result.push(byte_value);
            debug!(
                "Decoded font settings (weight={}, width={}, slant={}) to byte: {}",
                weight, width, slant, byte_value
            );
        }
    }

    if !result.is_empty() {
        info!(
            "Successfully decoded {} bytes from font variations",
            result.len()
        );
    } else {
        warn!("No data found in font variations");
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font() {
        let test_data = b"Hello, Font Steganography!";
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
    fn test_font_variation_encoding() {
        let byte = 123;
        let variation = byte_to_font_variation(byte, 0);
        assert!(variation.contains("font-variation-settings"));
        assert!(variation.contains("wght"));
        assert!(variation.contains("wdth"));
        assert!(variation.contains("slnt"));
    }
}
