/*!
 * Steganography module providing various data hiding implementations
 *
 * This module contains different steganographic techniques for hiding data in various
 * file formats and data structures. Key features include:
 *
 * - Audio steganography for hiding data in audio files
 * - CSS-based data hiding techniques
 * - Font-based steganography
 * - Grid-based data encoding
 * - HTML and web-based hiding methods
 * - JSON structure manipulation
 * - RSS feed steganography
 * - SVG path manipulation
 */

pub mod audio;
pub mod css;
pub mod font;
pub mod grid;
pub mod houdini;
pub mod html;
pub mod json;
pub mod prism;
pub mod rss;
pub mod svg_path;
pub mod xml;

use rand::Rng;
use tracing::{debug, warn};

use crate::Result;
use audio::AudioEncoder;

pub trait Encoder {
    fn name(&self) -> &'static str;
    fn encode(&self, data: &[u8]) -> Result<String>;
    fn decode(&self, content: &str) -> Result<Vec<u8>>;
}

const MIME_TYPES: &[(&str, &[&str])] = &[
    ("text/html", &["html", "prism", "font"]),
    ("text/css", &["css", "houdini", "grid"]),
    ("application/json", &["json"]),
    ("application/xml", &["xml", "rss"]),
    ("audio/wav", &["audio"]),
    ("image/svg+xml", &["svg_path"]),
];

/// Get random MIME type
pub fn get_random_mime_type() -> String {
    let (mime_type, _) = MIME_TYPES[rand::thread_rng().gen_range(0..MIME_TYPES.len())];
    mime_type.to_string()
}

/// Encode data based on MIME type
pub fn encode_mime(data: &[u8], mime_type: &str) -> Result<Vec<u8>> {
    debug!("Encoding data with MIME type: {}", mime_type);

    match mime_type {
        "text/html" => {
            // Randomly choose HTML, Prism, or Font encoder
            let choice = rand::thread_rng().gen_range(0..3);
            match choice {
                0 => html::encode(data),
                1 => prism::encode(data),
                _ => font::encode(data),
            }
        }
        "text/css" => {
            // Randomly choose CSS, Houdini, or Grid encoder
            let choice = rand::thread_rng().gen_range(0..3);
            match choice {
                0 => css::encode(data),
                1 => houdini::encode(data),
                _ => grid::encode(data),
            }
        }
        "application/json" => json::encode(data),
        "application/xml" => {
            // Randomly choose XML or RSS encoder
            if rand::thread_rng().gen_bool(0.5) {
                xml::encode(data)
            } else {
                rss::encode(data)
            }
        }
        "audio/wav" => {
            let encoder = AudioEncoder::default();
            let encoded = encoder.encode(data)?;
            Ok(encoded.into_bytes())
        }
        "image/svg+xml" => svg_path::encode(data),
        _ => {
            warn!("Unsupported MIME type: {}", mime_type);
            Ok(data.to_vec())
        }
    }
}

/// Decode data based on MIME type
pub fn decode_mime(data: &[u8], mime_type: &str) -> Result<Vec<u8>> {
    debug!("Decoding data with MIME type: {}", mime_type);

    match mime_type {
        "text/html" => {
            // Try HTML, Prism, and Font decoding
            match html::decode(data) {
                Ok(decoded) if !decoded.is_empty() => Ok(decoded),
                _ => match prism::decode(data) {
                    Ok(decoded) if !decoded.is_empty() => Ok(decoded),
                    _ => font::decode(data),
                },
            }
        }
        "text/css" => {
            // Try CSS, Houdini, and Grid decoding
            match css::decode(data) {
                Ok(decoded) if !decoded.is_empty() => Ok(decoded),
                _ => match houdini::decode(data) {
                    Ok(decoded) if !decoded.is_empty() => Ok(decoded),
                    _ => grid::decode(data),
                },
            }
        }
        "application/json" => json::decode(data),
        "application/xml" => {
            // Try XML decoding, if fails try RSS decoding
            match xml::decode(data) {
                Ok(decoded) if !decoded.is_empty() => Ok(decoded),
                _ => rss::decode(data),
            }
        }
        "audio/wav" => {
            let encoder = AudioEncoder::default();
            let content = String::from_utf8(data.to_vec())?;
            encoder.decode(&content)
        }
        "image/svg+xml" => svg_path::decode(data),
        _ => {
            warn!("Unsupported MIME type: {}", mime_type);
            Ok(data.to_vec())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mime_type_encoding() {
        let test_data = b"Hello, MIME Type Steganography!";

        // Test all MIME types
        for (mime_type, _) in MIME_TYPES {
            let encoded = encode_mime(test_data, mime_type).unwrap();
            let decoded = decode_mime(&encoded, mime_type).unwrap();
            assert_eq!(decoded, test_data);
        }
    }

    #[test]
    fn test_random_mime_type() {
        let mime_type = get_random_mime_type();
        assert!(MIME_TYPES.iter().any(|(mt, _)| *mt == mime_type));
    }

    #[test]
    fn test_unsupported_mime_type() {
        let test_data = b"Hello, Unsupported MIME Type!";
        let encoded = encode_mime(test_data, "unsupported/type").unwrap();
        let decoded = decode_mime(&encoded, "unsupported/type").unwrap();
        assert_eq!(decoded, test_data);
    }

    #[test]
    fn test_empty_data_mime() {
        let test_data = b"";
        for (mime_type, _) in MIME_TYPES {
            let encoded = encode_mime(test_data, mime_type).unwrap();
            let decoded = decode_mime(&encoded, mime_type).unwrap();
            assert!(decoded.is_empty());
        }
    }

    #[test]
    fn test_large_data_mime() {
        let test_data: Vec<u8> = (0..2000).map(|i| (i % 256) as u8).collect();
        for (mime_type, _) in MIME_TYPES {
            let encoded = encode_mime(&test_data, mime_type).unwrap();
            let decoded = decode_mime(&encoded, mime_type).unwrap();
            assert!(!decoded.is_empty());
        }
    }
}
