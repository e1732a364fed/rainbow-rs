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
    content_text: String,
    anim_prefix: String,
    elem_prefix: String,
    delay_one: std::ops::Range<f32>,
    delay_zero: std::ops::Range<f32>,
}

impl Random for CssEncoder {
    fn random() -> Self {
        let mut rng = thread_rng();

        // 随机生成两个不重叠的范围
        // delay_one 在 0.1..0.4 之间生成一个 0.1 宽度的范围
        let one_start = rng.gen_range(0.1..0.3);
        let delay_one = one_start..(one_start + 0.1);

        // delay_zero 在 0.5..0.8 之间生成一个 0.1 宽度的范围
        let zero_start = rng.gen_range(0.5..0.7);
        let delay_zero = zero_start..(zero_start + 0.1);

        Self {
            content_text: lorem::en::Paragraph(1..3).fake::<String>(),
            anim_prefix: format!(
                "anim-{}",
                name::en::FirstName().fake::<String>().to_lowercase()
            ),
            elem_prefix: format!(
                "elem-{}",
                name::en::LastName().fake::<String>().to_lowercase()
            ),
            delay_one,
            delay_zero,
        }
    }
}

impl Default for CssEncoder {
    fn default() -> Self {
        Self {
            content_text: "Experience smooth animations and transitions.".to_string(),
            anim_prefix: "a".to_string(),
            elem_prefix: "e".to_string(),
            delay_one: 0.1..0.3,
            delay_zero: 0.5..0.7,
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

/// Encode data into CSS animation. Output is a CSS file.
pub fn encode(
    data: &[u8],
    content_text: &str,
    anim_prefix: &str,
    elem_prefix: &str,
    delay_one: &std::ops::Range<f32>,
    delay_zero: &std::ops::Range<f32>,
) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Ok(content_text.as_bytes().to_vec());
    }

    let mut animations = vec![".content { font-family: Arial; line-height: 1.6; }".to_string()];
    let mut rng = thread_rng();

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
        let delays: Vec<String> = chunk_bits
            .iter()
            .map(|&bit| {
                let delay = if bit == 1 {
                    rng.gen_range(delay_one.clone())
                } else {
                    rng.gen_range(delay_zero.clone())
                };
                format!("{}s", delay)
            })
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
    }

    // Generate complete css
    let css = animations.join("\n");

    Ok(css.into_bytes())
}

/// Decode data from CSS animation
pub fn decode(data: &[u8]) -> Result<Vec<u8>> {
    let css = String::from_utf8_lossy(data);

    // Extract all animation delay times
    let re = regex::Regex::new(r"animation-delay:\s*([^;]+)").unwrap();
    let mut all_bits = Vec::new();

    for cap in re.captures_iter(&css) {
        let delays = cap.get(1).unwrap().as_str();
        let times: Vec<&str> = delays.split(',').map(|s| s.trim()).collect();

        // Collect all bits
        for time in times {
            // 移除 's' 后缀并解析为浮点数
            let value = time.trim_end_matches('s').parse::<f32>().unwrap_or(0.0);
            all_bits.push(if value < 0.4 { 1u8 } else { 0u8 });
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

    const TEST_DATA: &[u8] = b"Hello, CSS Animation Steganography!";

    #[test]
    fn test_encode_decode() {
        let encoded = encode(
            TEST_DATA,
            "Test content",
            "anim",
            "elem",
            &(0.1..0.3),
            &(0.5..0.7),
        )
        .unwrap();
        assert!(!encoded.is_empty());
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, TEST_DATA);

        let encoder = CssEncoder::default();
        let encoded = encoder.encode(TEST_DATA).unwrap();
        let decoded = encoder.decode(&encoded).unwrap();
        assert_eq!(decoded, TEST_DATA);
    }

    #[test]
    fn test_random() {
        let encoder = CssEncoder::random();
        let encoded = encoder.encode(TEST_DATA).unwrap();
        let decoded = encoder.decode(&encoded).unwrap();
        assert_eq!(decoded, TEST_DATA);
    }

    #[test]
    fn test_empty_data() {
        let encoded = encode(
            b"",
            "Test content",
            "anim",
            "elem",
            &(0.1..0.3),
            &(0.5..0.7),
        )
        .unwrap();
        assert!(!encoded.is_empty());
        let decoded = decode(&encoded);
        assert!(decoded.is_err());
    }
}
