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
use fake::{faker::*, Fake};
use regex::Regex;
use tracing::{debug, info, trace, warn};

use crate::stego::{Encoder, Random};

#[derive(Debug, Clone)]

pub struct FontEncoder {
    page_title: String,
    font_family: String,
    heading: String,
    tail_text: String,
}

impl Random for FontEncoder {
    fn random() -> Self {
        Self {
            page_title: format!(
                "Font Gallery - {}",
                company::en::CompanyName().fake::<String>()
            ),
            font_family: format!("{} Sans", name::en::LastName().fake::<String>()),
            heading: format!(
                "Typography by {}",
                company::en::CompanyName().fake::<String>()
            ),
            tail_text: lorem::en::Paragraph(2..4).fake::<String>(),
        }
    }
}

impl Default for FontEncoder {
    fn default() -> Self {
        Self {
            page_title: "Typography Showcase".to_string(),
            font_family: "Variable".to_string(),
            heading: "Typography Examples".to_string(),
            tail_text: "Exploring variable fonts in modern web design.".to_string(),
        }
    }
}

impl Encoder for FontEncoder {
    fn name(&self) -> &'static str {
        "font"
    }

    fn encode(&self, data: &[u8]) -> Result<Vec<u8>> {
        encode(
            data,
            &self.page_title,
            &self.font_family,
            &self.heading,
            &self.tail_text,
        )
    }

    fn decode(&self, content: &[u8]) -> Result<Vec<u8>> {
        decode(content)
    }

    fn get_mime_type(&self) -> &'static str {
        "text/html"
    }
}

/// Convert bytes to font variation settings
fn byte_to_font_variation(byte: u8, index: usize, font_family: &str) -> String {
    let weight = (byte / 16) as u32 * 100 + 100; // Weight range 100-1600
    let width = (byte % 16) as u32 * 6; // Width range 0-90
    let slant = (byte % 4) as u32 * 5; // Slant angles: 0, 5, 10, 15 degrees

    format!(
        r#".v{} {{
            font-variation-settings: 'wght' {}, 'wdth' {}, 'slnt' {};
            font-family: '{}';
        }}"#,
        index, weight, width, slant, font_family
    )
}

/// Generate complete HTML document
fn generate_html_document(
    variations: &[String],
    chars: &[String],
    page_title: &str,
    font_family: &str,
    heading: &str,
    tail_text: &str,
) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
    <style>
        @font-face {{
            font-family: '{}';
            src: url('data:font/woff2;base64,d09GMgABAAA...') format('woff2');
            font-weight: 100 900;
            font-stretch: 25% 151%;
            font-style: oblique 0deg 15deg;
        }}
        body {{
            font-family: '{}', sans-serif;
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
        <h1>{}</h1>
        {}
        <p>{}</p>
    </div>
</body>
</html>"#,
        page_title,
        font_family,
        font_family,
        variations.join("\n        "),
        heading,
        chars.join("\n        "),
        tail_text
    )
}

/// Encode data as font variations
pub fn encode(
    data: &[u8],
    page_title: &str,
    font_family: &str,
    heading: &str,
    tail_text: &str,
) -> Result<Vec<u8>> {
    debug!("Encoding data using font variation steganography");

    if data.is_empty() {
        return Ok(b"<!DOCTYPE html><html><head></head><body></body></html>".to_vec());
    }

    let mut variations = Vec::new();
    let mut chars = Vec::new();

    for (i, &byte) in data.iter().enumerate() {
        // Create font variation style
        variations.push(byte_to_font_variation(byte, i + 1, font_family));

        // Create character element with class
        chars.push(format!("<span class=\"v{}\">O</span>", i + 1));
    }

    debug!(
        "Generated font variation steganography with {} characters",
        data.len()
    );
    Ok(generate_html_document(
        &variations,
        &chars,
        page_title,
        font_family,
        heading,
        tail_text,
    )
    .into_bytes())
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
            trace!(
                "Decoded font settings (weight={}, width={}, slant={}) to byte: {}",
                weight,
                width,
                slant,
                byte_value
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
        let encoded = encode(
            test_data,
            "Test Typography",
            "TestFont",
            "Test Examples",
            "Test content for font variations.",
        )
        .unwrap();
        assert!(!encoded.is_empty());
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, test_data);
    }

    #[test]
    fn test_empty_data() {
        let test_data = b"";
        let encoded = encode(
            test_data,
            "Test Typography",
            "TestFont",
            "Test Examples",
            "Test content for font variations.",
        )
        .unwrap();
        assert!(!encoded.is_empty());
        let decoded = decode(&encoded).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_large_data() {
        let test_data: Vec<u8> = (0..2000).map(|i| (i % 256) as u8).collect();
        let encoded = encode(
            &test_data,
            "Test Typography",
            "TestFont",
            "Test Examples",
            "Test content for font variations.",
        )
        .unwrap();
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
        let variation = byte_to_font_variation(byte, 0, "Variable");
        assert!(variation.contains("font-variation-settings"));
        assert!(variation.contains("wght"));
        assert!(variation.contains("wdth"));
        assert!(variation.contains("slnt"));
    }
}
