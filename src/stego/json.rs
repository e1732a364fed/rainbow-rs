/*!
JSON Steganography Module

This module implements steganography by embedding data within JSON metadata fields.
The method works by:
- Converting input data to base64 encoding
- Embedding the encoded data into JSON metadata fields like timestamps, IDs, etc.
- Preserving valid JSON structure while hiding data

Key features:
- Maintains valid JSON format for stealth
- Uses common metadata fields to avoid suspicion
- Leverages base64 encoding for data compatibility

Use cases:
- Hiding data in JSON-based APIs and data exchanges
- Covert communication through JSON metadata
- Data embedding in JSON configuration files
*/

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::Utc;
use serde_json::{json, Value};
use tracing::{debug, info, warn};

use crate::Result;

/// Encode data into JSON metadata
pub fn encode(data: &[u8]) -> Result<Vec<u8>> {
    debug!("Encoding data using JSON metadata steganography");

    if data.is_empty() {
        return Ok(b"{}".to_vec());
    }

    // Encode data as Base64
    let encoded = BASE64.encode(data);

    // Build JSON document
    let json_obj = json!({
        "type": "metadata",
        "version": "1.0",
        "timestamp": Utc::now().timestamp(),
        "metadata": encoded,
        "description": "System configuration and metadata"
    });

    info!(
        "Generated JSON metadata steganography with {} bytes",
        data.len()
    );
    Ok(serde_json::to_vec(&json_obj)?)
}

/// Decode data from JSON metadata
pub fn decode(json_content: &[u8]) -> Result<Vec<u8>> {
    debug!("Decoding JSON metadata steganography");

    if json_content.is_empty() {
        warn!("Empty JSON content");
        return Ok(Vec::new());
    }

    // Log raw content for debugging
    debug!(
        "Raw JSON content: {}",
        String::from_utf8_lossy(json_content)
    );

    // Parse JSON
    let json_obj: Value = serde_json::from_slice(json_content)?;

    // Extract data from metadata field
    if let Some(encoded_data) = json_obj.get("metadata").and_then(|v| v.as_str()) {
        debug!("Found encoded data: {}", encoded_data);

        // Try to decode Base64 data
        if let Ok(decoded) = BASE64.decode(encoded_data) {
            info!(
                "Successfully decoded {} bytes from JSON metadata",
                decoded.len()
            );
            return Ok(decoded);
        }
    }

    warn!("No metadata field found in JSON content");
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json() {
        let test_data = b"Hello, JSON Steganography!";
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
        let result = decode(b"invalid content");
        assert!(result.is_err() || result.unwrap().is_empty());
    }
}
