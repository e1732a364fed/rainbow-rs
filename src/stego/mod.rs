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
pub mod lsb;
pub mod octet;
pub mod prism;
pub mod rss;
pub mod svg_path;
pub mod xml;

use std::collections::{HashMap, HashSet};

use rand::{seq::SliceRandom, Rng};
use tracing::debug;

use crate::{RainbowError, Result};
use audio::AudioEncoder;
use lsb::LSBEncoder;

/// A trait for types that can be randomly generated
pub trait Random {
    /// Create a new random instance of this type
    fn random() -> Self;
}

pub trait Encoder: std::fmt::Debug + dyn_clone::DynClone + Send + Sync {
    fn name(&self) -> &'static str;
    fn encode(&self, data: &[u8]) -> Result<Vec<u8>>;
    fn decode(&self, content: &[u8]) -> Result<Vec<u8>>;

    fn get_mime_type(&self) -> &'static str;
}

dyn_clone::clone_trait_object!(Encoder);

#[derive(Debug, Clone)]
pub struct EncoderRegistry {
    encoders: HashMap<String, Box<dyn Encoder>>,
}

impl Default for EncoderRegistry {
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
        encoders.insert("lsb".to_string(), Box::new(LSBEncoder::default()));
        encoders.insert(
            "svg_path".to_string(),
            Box::new(svg_path::SvgPathEncoder::default()),
        );
        encoders.insert(
            "octet".to_string(),
            Box::new(octet::OctetEncoder::default()),
        );
        Self { encoders }
    }
}

impl EncoderRegistry {
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
        encoders.insert("lsb".to_string(), Box::new(LSBEncoder::random()));
        encoders.insert(
            "svg_path".to_string(),
            Box::new(svg_path::SvgPathEncoder::random()),
        );
        encoders.insert("octet".to_string(), Box::new(octet::OctetEncoder::random()));
        Self { encoders }
    }

    pub fn get(&self, encoder: &str) -> Option<&dyn Encoder> {
        self.encoders.get(encoder).map(|encoder| encoder.as_ref())
    }

    pub fn add(&mut self, encoder: Box<dyn Encoder>) {
        self.encoders.insert(encoder.name().to_string(), encoder);
    }

    pub fn get_all_mime_types(&self) -> Vec<&str> {
        self.encoders
            .values()
            .map(|encoder| encoder.get_mime_type())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Get random MIME type
    pub fn get_random_mime_type(&self) -> String {
        let mime_types: Vec<&str> = self
            .encoders
            .values()
            .map(|encoder| encoder.get_mime_type())
            .collect();
        mime_types
            .choose(&mut rand::thread_rng())
            .unwrap()
            .to_string()
    }

    pub fn encode_with(&self, data: &[u8], encoder: &str) -> Result<Vec<u8>> {
        self.encoders
            .get(encoder)
            .ok_or(RainbowError::Other(format!(
                "Encoder not found: {}",
                encoder
            )))?
            .encode(data)
    }

    pub fn decode_with(&self, data: &[u8], decoder: &str) -> Result<Vec<u8>> {
        self.encoders
            .get(decoder)
            .ok_or(RainbowError::Other(format!(
                "Encoder not found: {}",
                decoder
            )))?
            .decode(data)
    }

    /// Encode data based on MIME type, will use a random encoder from the matching encoders
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

    /// Decode data based on MIME type, will try to use every matching encoder until one succeeds
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

        let mut last_error = None;

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
                r => {
                    last_error = Some(r);
                    continue;
                }
            }
        }

        // If no decoder succeeded, return the original data
        Err(RainbowError::Other(format!(
            "No decoder succeeded for MIME type: {}, last error: {:?}",
            mime_type, last_error
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

        let encoders = EncoderRegistry::default();

        // Test all MIME types
        for mime_type in encoders.get_all_mime_types() {
            let encoded = encoders.encode_mime(test_data, mime_type).unwrap();
            let decoded = encoders.decode_mime(&encoded, mime_type).unwrap();
            assert_eq!(decoded, test_data);
        }
    }

    #[test]
    fn test_random_mime_type() {
        let encoders = EncoderRegistry::default();
        let mime_type = encoders.get_random_mime_type();
        assert!(encoders.get_all_mime_types().contains(&mime_type.as_str()));
    }

    #[test]
    fn test_unsupported_mime_type() {
        let test_data = b"Hello, Unsupported MIME Type!";
        let encoders = EncoderRegistry::default();
        let encoded = encoders.encode_mime(test_data, "unsupported/type");
        assert!(encoded.is_err());
    }

    #[test]
    fn test_empty_data_mime() {
        init();
        let test_data = b"";
        let encoders = EncoderRegistry::default();
        for mime_type in encoders.get_all_mime_types() {
            let encoded = encoders.encode_mime(test_data, mime_type).unwrap();
            let decoded = encoders.decode_mime(&encoded, mime_type);

            debug!("Decoded: {:?}", decoded);

            assert!(decoded.is_err() || decoded.unwrap().is_empty());
        }
    }

    #[test]
    fn test_large_data_mime() {
        let test_data: Vec<u8> = (0..2000).map(|i| (i % 256) as u8).collect();
        let encoders = EncoderRegistry::default();
        for mime_type in encoders.get_all_mime_types() {
            let encoded = encoders.encode_mime(&test_data, mime_type).unwrap();
            let decoded = encoders.decode_mime(&encoded, mime_type).unwrap();
            assert!(!decoded.is_empty());
        }
    }
}
