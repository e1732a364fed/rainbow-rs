/*! CSS Grid/Flex Steganography Implementation

This module implements steganography using CSS Grid and Flex layout properties. The method works by:

- Encoding data bytes into CSS Grid/Flex property values like grid-template-columns and flex-grow
- Using valid CSS numeric values and units to maintain a natural appearance
- Leveraging the wide range of acceptable values in Grid/Flex layouts for data capacity

Key features:
- High capacity due to flexible numeric value ranges
- Natural appearance as common CSS layout properties
- Compatible with modern web pages using Grid/Flex layouts

Suitable for hiding data in CSS stylesheets where Grid/Flex layouts are expected.
*/

use crate::Result;
use fake::{faker::*, Fake};
use regex::Regex;
use tracing::debug;

use crate::stego::{Encoder, Random};

#[derive(Debug, Clone)]

pub struct GridEncoder {
    container_class: String,
}

impl Random for GridEncoder {
    fn random() -> Self {
        Self {
            container_class: format!(
                "grid-{}-{}",
                name::en::FirstName().fake::<String>().to_lowercase(),
                name::en::LastName().fake::<String>().to_lowercase()
            ),
        }
    }
}

impl Default for GridEncoder {
    fn default() -> Self {
        Self {
            container_class: "stego-container".to_string(),
        }
    }
}

impl Encoder for GridEncoder {
    fn name(&self) -> &'static str {
        "grid"
    }

    fn encode(&self, data: &[u8]) -> Result<Vec<u8>> {
        encode(data, &self.container_class)
    }

    fn decode(&self, content: &[u8]) -> Result<Vec<u8>> {
        decode(content)
    }

    fn get_mime_type(&self) -> &'static str {
        "text/css"
    }
}

/// Encode byte data as CSS Grid/Flex properties
pub fn encode(data: &[u8], container_class: &str) -> Result<Vec<u8>> {
    debug!("Encoding data using CSS Grid/Flex steganography");

    if data.is_empty() {
        return Ok(Vec::new());
    }

    let mut css = Vec::new();
    let mut grid_template = Vec::new();

    // Create container style
    css.push(format!(".{} {{", container_class));
    css.push("  display: grid;".to_string());
    css.push("  grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));".to_string());

    // Encode data using grid-gap and grid-template-areas
    let mut i = 0;
    while i < data.len() {
        // Use gap to encode first byte
        let gap = data[i];
        css.push(format!("  gap: {}px;", gap));

        // Use grid-template-areas to encode second byte
        if i + 1 < data.len() {
            let area_name = format!("a{}", data[i + 1]);
            grid_template.push(format!("\"{}\"", area_name));
        }

        i += 2;
    }

    // Add grid-template-areas
    if !grid_template.is_empty() {
        css.push(format!(
            "  grid-template-areas: {};",
            grid_template.join(" ")
        ));
    }
    css.push("}".to_string());

    debug!("Generated CSS Grid/Flex styles with {} bytes", data.len());
    Ok(css.join("\n").into_bytes())
}

/// Decode data from CSS Grid/Flex properties
pub fn decode(data: &[u8]) -> Result<Vec<u8>> {
    debug!("Decoding CSS Grid/Flex steganography");

    if data.is_empty() {
        return Ok(Vec::new());
    }

    let css = String::from_utf8_lossy(data);
    let mut bytes = Vec::new();

    // Extract data from gap values
    let gap_re = Regex::new(r"gap:\s*(\d+)px").unwrap();
    let gaps: Vec<u8> = gap_re
        .captures_iter(&css)
        .filter_map(|cap| cap[1].parse().ok())
        .collect();

    // Extract data from grid-template-areas
    let area_re = Regex::new(r#""a(\d+)""#).unwrap();
    let areas: Vec<u8> = area_re
        .captures_iter(&css)
        .filter_map(|cap| cap[1].parse().ok())
        .collect();

    // Rebuild byte array in original encoding order
    for i in 0..gaps.len() {
        bytes.push(gaps[i]);
        if i < areas.len() {
            bytes.push(areas[i]);
        }
    }

    debug!(
        "Successfully decoded {} bytes from CSS Grid/Flex styles",
        bytes.len()
    );
    Ok(bytes)
}

/// Check if the given data might contain steganographic content
pub fn detect(data: &[u8]) -> bool {
    if data.is_empty() {
        return false;
    }

    let css = String::from_utf8_lossy(data);
    let has_grid = css.contains("display: grid");
    let has_gap = Regex::new(r"gap:\s*\d+px").unwrap().is_match(&css);
    let has_areas = css.contains("grid-template-areas:");

    has_grid && has_gap && has_areas
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid() {
        let test_data = b"Hello, Grid Steganography!";
        let encoded = encode(test_data, "stego-container").unwrap();
        assert!(!encoded.is_empty());
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, test_data);
    }

    #[test]
    fn test_empty_data() {
        let test_data = b"";
        let encoded = encode(test_data, "stego-container").unwrap();
        assert!(encoded.is_empty());
        let decoded = decode(&encoded).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_large_data() {
        let test_data: Vec<u8> = (0..2000).map(|i| (i % 256) as u8).collect();
        let encoded = encode(&test_data, "stego-container").unwrap();
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
    fn test_detect() {
        let test_data = b"Hello, Grid!";
        let encoded = encode(test_data, "stego-container").unwrap();
        assert!(detect(&encoded));
        assert!(!detect(b"Regular CSS content"));
    }
}
