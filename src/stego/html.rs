/*! HTML Steganography Implementation

This module implements steganography using HTML document structure as the carrier.
The method works by:
- Embedding data within HTML template variations
- Using different HTML templates as carriers
- Encoding secret data using base64 before embedding
- Maintaining valid HTML structure while hiding data

Key features:
- Preserves valid HTML syntax
- Uses multiple template variations for better concealment
- Suitable for scenarios requiring HTML-based data hiding
*/

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::seq::SliceRandom;

use crate::stego::Encoder;
use crate::Result;

pub struct HtmlEncoder {
    page_title: String,
    heading: String,
    content: String,
}

impl Default for HtmlEncoder {
    fn default() -> Self {
        Self {
            page_title: "Welcome".to_string(),
            heading: "Welcome to our site".to_string(),
            content: "This is a sample page.".to_string(),
        }
    }
}

impl Encoder for HtmlEncoder {
    fn name(&self) -> &'static str {
        "html"
    }

    fn encode(&self, data: &[u8]) -> Result<Vec<u8>> {
        encode(data, &self.page_title, &self.heading, &self.content)
    }

    fn decode(&self, content: &[u8]) -> Result<Vec<u8>> {
        decode(content)
    }
}

const HTML_TEMPLATES: &[&str] = &[
    r#"<!DOCTYPE html>
<html>
<head>
    <title>{title}</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
</head>
<body>
    <div class="container">
        <h1>{heading}</h1>
        <p>{content}</p>
        <!-- {data} -->
    </div>
</body>
</html>"#,
    r#"<!DOCTYPE html>
<html>
<head>
    <title>{title}</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
</head>
<body>
    <article>
        <h1>{heading}</h1>
        <section>
            <!-- {data} -->
            <p>{content}</p>
        </section>
    </article>
</body>
</html>"#,
];

/// Encode data into HTML comments
pub fn encode(data: &[u8], page_title: &str, heading: &str, content: &str) -> Result<Vec<u8>> {
    // Handle empty data case
    if data.is_empty() {
        return Ok(b"<!DOCTYPE html><html><head></head><body></body></html>".to_vec());
    }

    let template = HTML_TEMPLATES.choose(&mut rand::thread_rng()).unwrap();
    let encoded = BASE64.encode(data);

    // Ensure encoded data doesn't contain "--" sequence
    let safe_encoded = encoded.replace("--", "-&#45;");
    let html = template
        .replace("{data}", &safe_encoded)
        .replace("{title}", page_title)
        .replace("{heading}", heading)
        .replace("{content}", content);

    Ok(html.into_bytes())
}

/// Decode data from HTML comments
pub fn decode(data: &[u8]) -> Result<Vec<u8>> {
    let html = String::from_utf8_lossy(data);

    if html.is_empty() {
        return Ok(Vec::new());
    }

    // Find data in comments using stricter pattern matching
    if let Some(start) = html.find("<!-- ") {
        if let Some(end) = html[start..].find(" -->") {
            let encoded = &html[start + 5..start + end];

            // Restore potentially escaped "--" sequences
            let restored = encoded.replace("-&#45;", "--");

            if let Ok(decoded) = BASE64.decode(restored) {
                return Ok(decoded);
            }
        }
    }

    // Return empty vector instead of original data if decoding fails
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html() {
        let test_data = b"Hello, HTML Steganography!";
        let encoded = encode(test_data, "Test Page", "Test Heading", "Test Content").unwrap();
        assert!(!encoded.is_empty());
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, test_data);
    }

    #[test]
    fn test_empty_data() {
        let test_data = b"";
        let encoded = encode(test_data, "Test Page", "Test Heading", "Test Content").unwrap();
        assert!(!encoded.is_empty());
        let decoded = decode(&encoded).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_large_data() {
        let test_data: Vec<u8> = (0..2000).map(|i| (i % 256) as u8).collect();
        let encoded = encode(&test_data, "Test Page", "Test Heading", "Test Content").unwrap();
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
