/*!
 * Houdini Steganography Implementation
 *
 * This module implements steganography using HTML/CSS painting parameters to hide data.
 * The method works by encoding secret messages into CSS styling properties like colors,
 * offsets and sizes that appear legitimate but contain hidden information.
 *
 * Key features:
 * - Hides data in CSS paint parameters that look like normal styling
 * - Uses color values, offsets and sizes as carriers for hidden bits
 * - Maintains visual appearance while storing secret data
 * - Suitable for web-based steganography scenarios
 */

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::Result;

#[derive(Debug, Serialize, Deserialize)]
struct PaintParam {
    color: String,
    offset: f64,
    size: f64,
}

/// Encode data as CSS Paint Worklet parameters
fn encode_to_paint_params(data: &[u8]) -> Vec<PaintParam> {
    let mut params = Vec::new();

    for (i, &byte) in data.iter().enumerate() {
        // Fix: Use bit shifting operations to get color components
        let r = (byte & 0xE0) >> 5; // Unchanged, take high 3 bits
        let g = (byte & 0x1C) >> 2; // Unchanged, take middle 3 bits
        let b = byte & 0x03; // Unchanged, take low 2 bits

        params.push(PaintParam {
            color: format!(
                "rgb({},{},{})",
                r * 32, // Correct: 0-7 maps to 0-224
                g * 32, // Correct: 0-7 maps to 0-224
                b * 64  // Correct: 0-3 maps to 0-192
            ),
            offset: (i as f64) * 0.1,
            size: 1.0 + (i % 3) as f64 * 0.5,
        });
    }

    params
}

/// Decode data from CSS Paint Worklet parameters
fn decode_from_paint_params(params: &[PaintParam]) -> Vec<u8> {
    let mut bytes = Vec::new();

    for param in params {
        let rgb_values: Vec<u8> = param
            .color
            .trim_start_matches("rgb(")
            .trim_end_matches(')')
            .split(',')
            .filter_map(|s| s.trim().parse::<u32>().ok())
            .map(|v| v as u8)
            .collect();

        if rgb_values.len() == 3 {
            // Fix: Map RGB values back to original scale
            let r = rgb_values[0] / 32; // 0-224 maps back to 0-7
            let g = rgb_values[1] / 32; // 0-224 maps back to 0-7
            let b = rgb_values[2] / 64; // 0-192 maps back to 0-3

            // Fix: Reconstruct byte using bit operations
            let byte = (r << 5) | (g << 2) | b;
            bytes.push(byte);
        }
    }

    bytes
}

/// Generate CSS Paint Worklet code
fn generate_paint_worklet() -> String {
    r#"if (typeof registerPaint !== 'undefined') {
    class StegoPainter {
        static get inputProperties() {
            return ['--stego-params'];
        }

        paint(ctx, size, properties) {
            const params = JSON.parse(properties.get('--stego-params'));
            params.forEach(param => {
                ctx.fillStyle = param.color;
                const x = size.width * param.offset;
                const y = size.height * param.offset;
                const s = param.size;
                ctx.fillRect(x, y, s, s);
            });
        }
    }
    registerPaint('stego-pattern', StegoPainter);
}"#
    .to_string()
}

/// Generate CSS style using Paint Worklet
fn generate_css_style(params: &[PaintParam]) -> Result<String> {
    let json_str = serde_json::to_string(params)?;
    Ok(format!(
        r#"@property --stego-params {{
    syntax: '*';
    inherits: false;
    initial-value: '{}';
}}
.stego-container {{
    --stego-params: '{}';
    background-image: paint(stego-pattern);
}}"#,
        json_str, json_str
    ))
}

/// Encode data to CSS Paint Worklet
pub fn encode(data: &[u8]) -> Result<Vec<u8>> {
    let params = encode_to_paint_params(data);
    let worklet = generate_paint_worklet();
    let style = generate_css_style(&params)?;

    let output = json!({
        "worklet": worklet,
        "style": style,
        "params": params,
    });

    Ok(serde_json::to_vec(&output)?)
}

/// Decode data from CSS Paint Worklet
pub fn decode(data: &[u8]) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Ok(Vec::new());
    }

    let json = match serde_json::from_slice::<serde_json::Value>(data) {
        Ok(v) => v,
        Err(_) => return Ok(Vec::new()),
    };

    if let Some(params) = json.get("params") {
        let params: Vec<PaintParam> = match serde_json::from_value(params.clone()) {
            Ok(v) => v,
            Err(_) => return Ok(Vec::new()),
        };
        Ok(decode_from_paint_params(&params))
    } else {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_houdini() {
        let test_data = b"Hello, Houdini Steganography!";
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
    fn test_paint_param_encoding() {
        let test_data = b"Test";
        let params = encode_to_paint_params(test_data);
        assert!(!params.is_empty());
        let decoded = decode_from_paint_params(&params);
        assert_eq!(decoded, test_data);
    }
}
