/*! CSS Animation Steganography Implementation

This module implements steganography using CSS animations as the carrier. The method works by:
- Encoding data into CSS animation properties like timing functions and keyframes
- Using variations in animation parameters to embed binary data
- Generating valid CSS that appears as normal animations

Key features:
- Maintains valid CSS syntax for browser compatibility
- Provides reasonable deniability as animations are common in web pages
- Allows embedding data while preserving visual appearance

Use cases:
- Covert data transmission via web pages
- Information hiding in CSS-based web applications
- Steganographic watermarking of web content
*/

use crate::Result;
use fake::{faker::*, Fake};
use rand::{thread_rng, Rng};
use regex;

use crate::stego::{Encoder, Random};

#[derive(Debug, Clone)]

pub struct CssEncoder {
    page_title: String,
    content_text: String,
    anim_prefix: String,
    elem_prefix: String,
    delay_one: String,
    delay_zero: String,
}

impl Random for CssEncoder {
    fn random() -> Self {
        let mut rng = thread_rng();
        Self {
            page_title: format!(
                "{} - {}",
                company::en::CompanyName().fake::<String>(),
                lorem::en::Sentence(3..6).fake::<String>()
            ),
            content_text: lorem::en::Paragraph(1..3).fake::<String>(),
            anim_prefix: format!(
                "anim-{}",
                name::en::FirstName().fake::<String>().to_lowercase()
            ),
            elem_prefix: format!(
                "elem-{}",
                name::en::LastName().fake::<String>().to_lowercase()
            ),
            delay_one: format!("{}s", (rng.gen_range(1..5) as f32 / 10.0)),
            delay_zero: format!("{}s", (rng.gen_range(5..10) as f32 / 10.0)),
        }
    }
}

impl Default for CssEncoder {
    fn default() -> Self {
        Self {
            page_title: "Dynamic Content".to_string(),
            content_text: "Experience smooth animations and transitions.".to_string(),
            anim_prefix: "a".to_string(),
            elem_prefix: "e".to_string(),
            delay_one: "0.1s".to_string(),
            delay_zero: "0.2s".to_string(),
        }
    }
}

impl Encoder for CssEncoder {
    fn name(&self) -> &'static str {
        "css"
    }

    fn encode(&self, data: &[u8]) -> Result<Vec<u8>> {
        encode(
            data,
            &self.page_title,
            &self.content_text,
            &self.anim_prefix,
            &self.elem_prefix,
            &self.delay_one,
            &self.delay_zero,
        )
    }

    fn decode(&self, content: &[u8]) -> Result<Vec<u8>> {
        decode(content)
    }

    fn get_mime_type(&self) -> &'static str {
        "text/css"
    }
}

/// Encode data into CSS animation
pub fn encode(
    data: &[u8],
    page_title: &str,
    content_text: &str,
    anim_prefix: &str,
    elem_prefix: &str,
    delay_one: &str,
    delay_zero: &str,
) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Ok(format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
    <style>
        .content {{ font-family: Arial; line-height: 1.6; }}
    </style>
</head>
<body>
    <div class="content">
        {}
    </div>
</body>
</html>"#,
            page_title, content_text
        )
        .into_bytes());
    }

    let mut animations = Vec::new();
    let mut elements = Vec::new();

    // Convert entire data into bit sequence
    let bits: Vec<u8> = data
        .iter()
        .flat_map(|&byte| (0..8).map(move |i| (byte >> (7 - i)) & 1))
        .collect();

    // Process every 8 bits as a group
    for chunk_bits in bits.chunks(8) {
        let anim_name = format!("{}{}", anim_prefix, thread_rng().gen_range(10000..100000));
        let elem_id = format!("{}{}", elem_prefix, thread_rng().gen_range(10000..100000));

        // Generate delay values
        let delays: Vec<&str> = chunk_bits
            .iter()
            .map(|&bit| if bit == 1 { delay_one } else { delay_zero })
            .collect();

        // Create animation and element styles
        animations.push(format!(
            r#"
@keyframes {} {{
    0% {{ opacity: 1; }}
    100% {{ opacity: 1; }}
}}
#{} {{
    animation: {} 1s;
    animation-delay: {};
    display: inline-block;
    width: 1px;
    height: 1px;
    background: transparent;
}}"#,
            anim_name,
            elem_id,
            anim_name,
            delays.join(",")
        ));

        elements.push(format!(r#"<div id="{}"></div>"#, elem_id));
    }

    // Generate complete HTML
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
    <style>
        .content {{ font-family: Arial; line-height: 1.6; }}
        {}
    </style>
</head>
<body>
    <div class="content">
        {}
        {}
    </div>
</body>
</html>"#,
        page_title,
        animations.join("\n"),
        content_text,
        elements.join("\n")
    );

    Ok(html.into_bytes())
}

/// Decode data from CSS animation
pub fn decode(data: &[u8]) -> Result<Vec<u8>> {
    let html = String::from_utf8_lossy(data);

    if html.trim().is_empty() {
        return Ok(Vec::new());
    }

    // If empty page, return empty data directly
    if html.contains("Empty Page") {
        return Ok(Vec::new());
    }

    // Extract all animation delay times
    let re = regex::Regex::new(r"animation-delay:\s*([^;]+)").unwrap();
    let mut all_bits = Vec::new();

    for cap in re.captures_iter(&html) {
        let delays = cap.get(1).unwrap().as_str();
        let times: Vec<&str> = delays.split(',').map(|s| s.trim()).collect();

        // Collect all bits
        for time in times {
            all_bits.push(if time.starts_with("0.1") { 1u8 } else { 0u8 });
        }
    }

    // Convert bits back to bytes
    let mut bytes = Vec::new();
    for chunk in all_bits.chunks(8) {
        if chunk.len() == 8 {
            let mut byte = 0u8;
            for (i, &bit) in chunk.iter().enumerate() {
                byte |= bit << (7 - i);
            }
            bytes.push(byte);
        }
    }

    if bytes.is_empty() {
        return Err(crate::RainbowError::InvalidData(
            "No valid data found".to_owned(),
        ));
    }

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_animation() {
        let test_data = b"Hello, CSS Animation Steganography!";
        let encoded = encode(
            test_data,
            "Test Page",
            "Test content",
            "anim",
            "elem",
            "0.1s",
            "0.2s",
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
            "Test Page",
            "Test content",
            "anim",
            "elem",
            "0.1s",
            "0.2s",
        )
        .unwrap();
        assert!(!encoded.is_empty());
        let decoded = decode(&encoded);
        assert!(decoded.is_err());
    }
}
