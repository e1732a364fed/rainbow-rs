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

use std::collections::HashMap;

use dyn_clone::DynClone;
use rand::{seq::SliceRandom, Rng};
use tracing::debug;

use crate::{RainbowError, Result};
use audio::AudioEncoder;

/// A trait for types that can be randomly generated
pub trait Random {
    /// Create a new random instance of this type
    fn random() -> Self;
}

pub trait Encoder: std::fmt::Debug + DynClone {
    fn name(&self) -> &'static str;
    fn encode(&self, data: &[u8]) -> Result<Vec<u8>>;
    fn decode(&self, content: &[u8]) -> Result<Vec<u8>>;

    fn get_mime_type(&self) -> &'static str;
}

dyn_clone::clone_trait_object!(Encoder);

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

#[derive(Debug, Clone)]
pub struct EncodersHolder {
    encoders: HashMap<String, Box<dyn Encoder>>,
}

impl Default for EncodersHolder {
    fn default() -> Self {
        let mut encoders: HashMap<String, Box<dyn Encoder>> = HashMap::new();
        encoders.insert("html".to_string(), Box::new(html::HtmlEncoder::default()));
        encoders.insert("json".to_string(), Box::new(json::JsonEncoder::default()));

        encoders.insert(
            "prism".to_string(),
            Box::new(prism::PrismEncoder::default()),
        );
        encoders.insert("font".to_string(), Box::new(font::FontEncoder::default()));
        encoders.insert("css".to_string(), Box::new(css::CssEncoder::default()));
        encoders.insert(
            "houdini".to_string(),
            Box::new(houdini::HoudiniEncoder::default()),
        );
        encoders.insert("grid".to_string(), Box::new(grid::GridEncoder::default()));
        encoders.insert("xml".to_string(), Box::new(xml::XmlEncoder::default()));
        encoders.insert("rss".to_string(), Box::new(rss::RssEncoder::default()));
        encoders.insert("audio".to_string(), Box::new(AudioEncoder::default()));
        encoders.insert(
            "svg_path".to_string(),
            Box::new(svg_path::SvgPathEncoder::default()),
        );
        Self { encoders }
    }
}

impl EncodersHolder {
    pub fn new_randomized() -> Self {
        let mut encoders: HashMap<String, Box<dyn Encoder>> = HashMap::new();
        encoders.insert("html".to_string(), Box::new(html::HtmlEncoder::random()));
        encoders.insert("json".to_string(), Box::new(json::JsonEncoder::random()));
        encoders.insert("prism".to_string(), Box::new(prism::PrismEncoder::random()));
        encoders.insert("font".to_string(), Box::new(font::FontEncoder::random()));
        encoders.insert("css".to_string(), Box::new(css::CssEncoder::random()));
        encoders.insert(
            "houdini".to_string(),
            Box::new(houdini::HoudiniEncoder::random()),
        );
        encoders.insert("grid".to_string(), Box::new(grid::GridEncoder::random()));
        encoders.insert("xml".to_string(), Box::new(xml::XmlEncoder::random()));
        encoders.insert("rss".to_string(), Box::new(rss::RssEncoder::random()));
        encoders.insert("audio".to_string(), Box::new(AudioEncoder::default()));
        encoders.insert(
            "svg_path".to_string(),
            Box::new(svg_path::SvgPathEncoder::random()),
        );
        Self { encoders }
    }

    pub fn get(&self, encoder: &str) -> Option<&Box<dyn Encoder>> {
        self.encoders.get(encoder)
    }

    pub fn add(&mut self, encoder: Box<dyn Encoder>) {
        self.encoders.insert(encoder.name().to_string(), encoder);
    }
}

impl EncodersHolder {
    /// Encode data based on MIME type
    pub fn encode_mime(&self, data: &[u8], mime_type: &str) -> Result<Vec<u8>> {
        // Get all encoders that match the MIME type
        let matching_encoders: Vec<_> = self
            .encoders
            .iter()
            .filter(|(_, encoder)| encoder.get_mime_type() == mime_type)
            .collect();

        if matching_encoders.is_empty() {
            return Err(RainbowError::Other(format!(
                "Unsupported MIME type: {}",
                mime_type
            )));
        }

        // Randomly choose one of the matching encoders
        let (name, encoder) =
            matching_encoders[rand::thread_rng().gen_range(0..matching_encoders.len())];

        debug!(
            "Encoding data with MIME type: {} using encoder: {}",
            mime_type, name
        );

        encoder.encode(data)
    }

    /// Decode data based on MIME type
    pub fn decode_mime(&self, data: &[u8], mime_type: &str) -> Result<Vec<u8>> {
        // Get all encoders that match the MIME type
        let mut matching_encoders: Vec<_> = self
            .encoders
            .iter()
            .filter(|(_, encoder)| encoder.get_mime_type() == mime_type)
            .collect();

        if matching_encoders.is_empty() {
            return Err(RainbowError::Other(format!(
                "Unsupported MIME type: {}",
                mime_type
            )));
        }

        matching_encoders.shuffle(&mut rand::thread_rng());

        // Try each matching encoder until one succeeds
        for (name, encoder) in matching_encoders {
            debug!(
                "Decoding data with MIME type: {} using encoder: {}",
                mime_type, name
            );
            match encoder.decode(data) {
                Ok(decoded) if !decoded.is_empty() => {
                    debug!(
                        "Decoded data with MIME type: {} using encoder: {}",
                        mime_type, name
                    );
                    return Ok(decoded);
                }
                _ => continue,
            }
        }

        // If no decoder succeeded, return the original data
        Err(RainbowError::Other(format!(
            "No decoder succeeded for MIME type: {}",
            mime_type
        )))
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();
    }

    #[test]
    fn test_mime_type_encoding() {
        let test_data = b"Hello, MIME Type Steganography!";

        init();

        let encoders = EncodersHolder::default();

        // Test all MIME types
        for (mime_type, _) in MIME_TYPES {
            let encoded = encoders.encode_mime(test_data, mime_type).unwrap();
            let decoded = encoders.decode_mime(&encoded, mime_type).unwrap();
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
        let encoders = EncodersHolder::default();
        let encoded = encoders.encode_mime(test_data, "unsupported/type");
        assert!(encoded.is_err());
    }

    #[test]
    fn test_empty_data_mime() {
        init();
        let test_data = b"";
        let encoders = EncodersHolder::default();
        for (mime_type, _) in MIME_TYPES {
            let encoded = encoders.encode_mime(test_data, mime_type).unwrap();
            let decoded = encoders.decode_mime(&encoded, mime_type);

            debug!("Decoded: {:?}", decoded);

            assert!(decoded.is_err() || decoded.unwrap().is_empty());
        }
    }

    #[test]
    fn test_large_data_mime() {
        let test_data: Vec<u8> = (0..2000).map(|i| (i % 256) as u8).collect();
        let encoders = EncodersHolder::default();
        for (mime_type, _) in MIME_TYPES {
            let encoded = encoders.encode_mime(&test_data, mime_type).unwrap();
            let decoded = encoders.decode_mime(&encoded, mime_type).unwrap();
            assert!(!decoded.is_empty());
        }
    }
}
