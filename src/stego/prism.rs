/*!
 * PRISM Steganography Implementation
 *
 * This module implements the PRISM (Pseudo-Random Injection Steganographic Method) technique
 * which hides data within nested HTML div elements. The method works by:
 *
 * - Encoding the secret message into base64
 * - Creating multiple layers of nested <div> elements (between 20-250 layers)
 * - Injecting the encoded data into specific div attributes using pseudo-random distribution
 *
 * Key features:
 * - HTML-based steganography making it suitable for web contexts
 * - Variable number of layers provides additional security
 * - Uses base64 encoding for data preparation
 * - Pseudo-random distribution helps avoid detection
 *
 * Best used for:
 * - Web-based covert communication
 * - Hiding data in HTML documents
 * - Scenarios where HTML manipulation won't raise suspicion
 */

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::Rng;
use tracing::{debug, info};

use crate::Result;

const MIN_LAYERS: usize = 20;
const MAX_LAYERS: usize = 250;

/// Encode using nested HTML divs
pub fn encode(data: &[u8]) -> Result<Vec<u8>> {
    let encoded = BASE64.encode(data);
    debug!("Encoding {} bytes using Prism steganography", data.len());

    let mut rng = rand::thread_rng();
    let mut html = String::new();

    // Add HTML header
    html.push_str("<!DOCTYPE html>\n<html>\n<head>\n<title>Page Title</title>\n</head>\n<body>\n");
    html.push_str("    <div class=\"container\">\n");

    // Create a nested div structure for each character
    for c in encoded.chars() {
        let layers = rng.gen_range(MIN_LAYERS..=MAX_LAYERS);
        let mut div = String::new();

        // Create nested div structure
        for i in 1..=layers {
            div.push_str(&format!("<div class=\"l{}\">", i));
        }

        // Add character
        div.push(c);

        // Close all divs
        for _ in 1..=layers {
            div.push_str("</div>");
        }

        html.push_str("        ");
        html.push_str(&div);
        html.push('\n');
    }

    // Add HTML footer
    html.push_str("    </div>\n</body>\n</html>\n");

    info!(
        "Generated Prism steganography with {} nested divs",
        encoded.len()
    );
    Ok(html.into_bytes())
}

/// Decode data from nested HTML divs
pub fn decode(data: &[u8]) -> Result<Vec<u8>> {
    let html = String::from_utf8_lossy(data);
    debug!("Decoding Prism steganography from {} bytes", data.len());

    let mut encoded = String::new();

    // Extract character from each nested div structure
    for line in html.lines() {
        let line = line.trim();
        if line.starts_with("<div class=\"l1\">") {
            // Find innermost text content
            let mut depth = 0;
            let mut in_tag = false;
            let mut found_char = None;

            for (i, c) in line.chars().enumerate() {
                match c {
                    '<' => {
                        in_tag = true;
                        if line[i..].starts_with("<div") {
                            depth += 1;
                        } else if line[i..].starts_with("</div") {
                            depth -= 1;
                        }
                    }
                    '>' => {
                        in_tag = false;
                    }
                    c if !in_tag && depth > 0 && !c.is_whitespace() => {
                        found_char = Some(c);
                        break;
                    }
                    _ => {}
                }
            }

            if let Some(c) = found_char {
                encoded.push(c);
            }
        }
    }

    if encoded.is_empty() {
        return Ok(Vec::new());
    }

    // Base64 decode
    match BASE64.decode(&encoded) {
        Ok(decoded) => {
            info!(
                "Successfully decoded {} bytes from Prism steganography",
                decoded.len()
            );
            Ok(decoded)
        }
        Err(_) => Ok(Vec::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prism() {
        let test_data = b"Hello, Prism Steganography!";
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
}
