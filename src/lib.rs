/*!
 * Rainbow Steganography Library
 *
 * This library provides functionality for steganography operations.
 * It implements various steganographic techniques for hiding and extracting data within
 * different types of carrier files.
 *
 * Main components:
 * - rainbow: Implementation of [`NetworkSteganographyProcessor`]
 * - stego: Core steganography algorithms and traits
 * - utils: Common utility functions and helpers
 */

use async_trait::async_trait;
use dyn_clone::DynClone;
use thiserror::Error;

pub mod rainbow;
pub mod stego;
pub mod utils;

#[derive(Error, Debug)]
pub enum RainbowError {
    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Encode failed: {0}")]
    EncodeFailed(String),

    #[error("Decode failed: {0}")]
    DecodeFailed(String),

    #[error("Length mismatch: {0}")]
    LengthMismatch(String),

    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Base64 decode error: {0}")]
    Base64Error(#[from] base64::DecodeError),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Other: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, RainbowError>;

pub struct DecodeResult {
    pub data: Vec<u8>,
    pub expected_return_length: usize,
    pub is_read_end: bool,
}

pub struct EncodeResult {
    pub encoded_packets: Vec<Vec<u8>>,
    pub expected_return_packet_lengths: Vec<usize>,
}

#[async_trait]
pub trait NetworkSteganographyProcessor: Send + Sync + DynClone {
    async fn encode_write(
        &self,
        plain_data: &[u8],
        is_client: bool,
        mime_type: Option<String>,
    ) -> Result<EncodeResult>;

    async fn decrypt_single_read(
        &self,
        cipher_data: Vec<u8>,
        packet_index: usize,
        is_client: bool,
    ) -> Result<DecodeResult>;
}
dyn_clone::clone_trait_object!(NetworkSteganographyProcessor);

impl From<std::string::FromUtf8Error> for RainbowError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        RainbowError::Other(err.to_string())
    }
}
