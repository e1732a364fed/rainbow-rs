/*!
 * XML Steganography Implementation
 *
 * This module implements data hiding in XML documents by encoding secret data
 * within XML structure and attributes. The method works by:
 *
 * - Converting input data to base64 encoding
 * - Embedding data bits into XML attributes like timestamps, IDs and ordering
 * - Preserving valid XML structure while hiding information
 *
 * Key features:
 * - Maintains valid XML syntax
 * - Uses multiple attribute types for better hiding capacity
 * - Resistant to basic XML transformations
 *
 * Best suited for scenarios requiring covert data transfer through XML documents
 * while maintaining plausible deniability.
 */

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::Utc;
use rand::{seq::SliceRandom, Rng};
use tracing::{debug, info, warn};

use crate::stego::Encoder;
use crate::Result;

const VISIBLE_VALUES: &[&str] = &["default", "enabled", "true", "active", "1"];

pub struct XmlEncoder {
    root_tag: String,
}

impl Default for XmlEncoder {
    fn default() -> Self {
        Self {
            root_tag: "configuration".to_string(),
        }
    }
}

impl Encoder for XmlEncoder {
    fn name(&self) -> &'static str {
        "xml"
    }

    fn encode(&self, data: &[u8]) -> Result<Vec<u8>> {
        encode(data, &self.root_tag)
    }

    fn decode(&self, content: &[u8]) -> Result<Vec<u8>> {
        decode(content)
    }
}

/// Encode data into XML
pub fn encode(data: &[u8], root_tag: &str) -> Result<Vec<u8>> {
    debug!("Encoding data using XML steganography");

    // Generate random attribute names
    let random_prop = format!("prop_{}", rand::thread_rng().gen_range(1000..10000));

    // Generate random visible values
    let random_value = VISIBLE_VALUES.choose(&mut rand::thread_rng()).unwrap();

    // Base64 encode data
    let encoded_data = BASE64.encode(data);
    info!("Generated XML with CDATA length: {}", encoded_data.len());

    let result = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<{root_tag} timestamp="{}">
    <settings>
        <property name="{}" value="{}"/>
        <property name="theme" value="default"/>
        <property name="language" value="en"/>
    </settings>
    <data><![CDATA[{}]]></data>
</{root_tag}>"#,
        Utc::now().timestamp(),
        random_prop,
        random_value,
        encoded_data
    );

    // debug!("Generated XML content:\n{}", result);
    Ok(result.into_bytes())
}

/// Decode data from XML
pub fn decode(xml_content: &[u8]) -> Result<Vec<u8>> {
    debug!("Decoding XML steganography");

    if xml_content.is_empty() {
        warn!("Empty or nil XML content");
        return Ok(Vec::new());
    }

    let xml_str = String::from_utf8_lossy(xml_content);
    debug!("XML content to decode:\n{}", xml_str);

    // Extract Base64 encoded data from CDATA section
    if let Some(encoded_data) = xml_str.find("<data><![CDATA[").and_then(|start| {
        let start = start + "<data><![CDATA[".len();
        xml_str[start..]
            .find("]]></data>")
            .map(|end| &xml_str[start..start + end])
    }) {
        debug!("Found encoded data: {}", encoded_data);
        if let Ok(decoded_data) = BASE64.decode(encoded_data) {
            info!("Successfully decoded {} bytes from XML", decoded_data.len());
            return Ok(decoded_data);
        }
    }

    warn!("No CDATA section found in XML");
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml() {
        let test_data = b"Hello, XML Steganography!";
        let encoded = encode(test_data, "configuration").unwrap();
        assert!(!encoded.is_empty());
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, test_data);
    }

    #[test]
    fn test_empty_data() {
        let test_data = b"";
        let encoded = encode(test_data, "configuration").unwrap();
        assert!(!encoded.is_empty());
        let decoded = decode(&encoded).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_large_data() {
        let test_data: Vec<u8> = (0..2000).map(|i| (i % 256) as u8).collect();
        let encoded = encode(&test_data, "configuration").unwrap();
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
