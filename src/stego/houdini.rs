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

use fake::{Fake, Faker};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::stego::{Encoder, Random};
use crate::Result;

#[derive(Debug, Clone)]

pub struct HoudiniEncoder {
    worklet_name: String,
    class_name: String,
    property_name: String,
    painter_class_name: String,
}

impl Random for HoudiniEncoder {
    fn random() -> Self {
        Self {
            worklet_name: format!("paint-{}", Faker.fake::<String>().to_lowercase()),
            class_name: format!("container-{}", Faker.fake::<String>().to_lowercase()),
            property_name: format!("--param-{}", Faker.fake::<String>().to_lowercase()),
            painter_class_name: format!("Painter{}", Faker.fake::<String>()),
        }
    }
}

impl Default for HoudiniEncoder {
    fn default() -> Self {
        Self {
            worklet_name: "paint".to_string(),
            class_name: "container".to_string(),
            property_name: "--params".to_string(),
            painter_class_name: "Painter".to_string(),
        }
    }
}

impl Encoder for HoudiniEncoder {
    fn name(&self) -> &'static str {
        "houdini"
    }

    fn encode(&self, data: &[u8]) -> Result<Vec<u8>> {
        encode(
            data,
            &self.worklet_name,
            &self.class_name,
            &self.property_name,
            &self.painter_class_name,
        )
    }

    fn decode(&self, content: &[u8]) -> Result<Vec<u8>> {
        decode(content)
    }

    fn get_mime_type(&self) -> &'static str {
        "text/css"
    }
}

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
fn generate_paint_worklet(
    worklet_name: &str,
    property_name: &str,
    painter_class_name: &str,
) -> String {
    format!(
        r#"if (typeof registerPaint !== 'undefined') {{
    class {} {{
        static get inputProperties() {{
            return ['{}'];
        }}

        paint(ctx, size, properties) {{
            const params = JSON.parse(properties.get('{}'));
            params.forEach(param => {{
                ctx.fillStyle = param.color;
                const x = size.width * param.offset;
                const y = size.height * param.offset;
                const s = param.size;
                ctx.fillRect(x, y, s, s);
            }});
        }}
    }}
    registerPaint('{}', {});
}}"#,
        painter_class_name, property_name, property_name, worklet_name, painter_class_name
    )
}

/// Generate CSS style using Paint Worklet
fn generate_css_style(
    params: &[PaintParam],
    class_name: &str,
    property_name: &str,
    worklet_name: &str,
) -> Result<String> {
    let json_str = serde_json::to_string(params)?;
    Ok(format!(
        r#"@property {} {{
    syntax: '*';
    inherits: false;
    initial-value: '{}';
}}
.{} {{
    {}: '{}';
    background-image: paint({});
}}"#,
        property_name, json_str, class_name, property_name, json_str, worklet_name
    ))
}

/// Encode data to CSS Paint Worklet
pub fn encode(
    data: &[u8],
    worklet_name: &str,
    class_name: &str,
    property_name: &str,
    painter_class_name: &str,
) -> Result<Vec<u8>> {
    let params = encode_to_paint_params(data);
    let worklet = generate_paint_worklet(worklet_name, property_name, painter_class_name);
    let style = generate_css_style(&params, class_name, property_name, worklet_name)?;

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
        let encoded = encode(
            test_data,
            "stego-paint",
            "stego-container",
            "--stego-params",
            "CustomPainter",
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
            "stego-paint",
            "stego-container",
            "--stego-params",
            "CustomPainter",
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
            "stego-paint",
            "stego-container",
            "--stego-params",
            "CustomPainter",
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
    fn test_paint_param_encoding() {
        let test_data = b"Test";
        let params = encode_to_paint_params(test_data);
        assert!(!params.is_empty());
        let decoded = decode_from_paint_params(&params);
        assert_eq!(decoded, test_data);
    }
}
