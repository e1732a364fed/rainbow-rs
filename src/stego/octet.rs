use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key};
use rand::RngCore;
use tracing::debug;

use super::{Encoder, Random};
use crate::{RainbowError, Result};

/// Encryption method supported by OctetEncoder
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EncryptionMethod {
    /// AES-256-GCM
    Aes,
    /// ChaCha20-Poly1305
    ChaCha,
}

impl Default for EncryptionMethod {
    fn default() -> Self {
        Self::ChaCha
    }
}

/// OctetEncoder implements steganography for application/octet-stream MIME type
/// It encrypts data using either AES-GCM or ChaCha20-Poly1305
#[derive(Debug, Clone)]
pub struct OctetEncoder {
    /// Encryption method to use
    method: EncryptionMethod,
    /// Encryption key (32 bytes for both AES-256-GCM and ChaCha20-Poly1305)
    key: [u8; 32],
}

impl Default for OctetEncoder {
    fn default() -> Self {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        Self {
            method: EncryptionMethod::default(),
            key,
        }
    }
}

impl Random for OctetEncoder {
    fn random() -> Self {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        Self {
            method: if rand::random() {
                EncryptionMethod::Aes
            } else {
                EncryptionMethod::ChaCha
            },
            key,
        }
    }
}

impl OctetEncoder {
    /// Create a new OctetEncoder with specified method and key
    pub fn new(method: EncryptionMethod, key: [u8; 32]) -> Self {
        Self { method, key }
    }

    fn encrypt_aes(&self, data: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>> {
        let cipher = Aes256Gcm::new(self.key.as_slice().into());
        let nonce = Nonce::from_slice(nonce);
        cipher
            .encrypt(nonce, data)
            .map_err(|e| RainbowError::Other(format!("AES encryption failed: {}", e)))
    }

    fn decrypt_aes(&self, data: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>> {
        let cipher = Aes256Gcm::new(self.key.as_slice().into());
        let nonce = Nonce::from_slice(nonce);
        cipher
            .decrypt(nonce, data)
            .map_err(|e| RainbowError::Other(format!("AES decryption failed: {}", e)))
    }

    fn encrypt_chacha(&self, data: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>> {
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&self.key));
        let nonce = chacha20poly1305::Nonce::from_slice(nonce);
        cipher
            .encrypt(nonce, data)
            .map_err(|e| RainbowError::Other(format!("ChaCha encryption failed: {}", e)))
    }

    fn decrypt_chacha(&self, data: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>> {
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&self.key));
        let nonce = chacha20poly1305::Nonce::from_slice(nonce);
        cipher
            .decrypt(nonce, data)
            .map_err(|e| RainbowError::Other(format!("ChaCha decryption failed: {}", e)))
    }
}

impl Encoder for OctetEncoder {
    fn name(&self) -> &'static str {
        "octet"
    }

    fn get_mime_type(&self) -> &'static str {
        "application/octet-stream"
    }

    fn encode(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut rng = rand::thread_rng();

        // Generate random nonce
        let mut nonce = [0u8; 12];
        rng.fill_bytes(&mut nonce);

        // Encrypt the data
        let encrypted = match self.method {
            EncryptionMethod::Aes => self.encrypt_aes(data, &nonce)?,
            EncryptionMethod::ChaCha => self.encrypt_chacha(data, &nonce)?,
        };

        // Create output buffer with exact size
        let mut output = Vec::with_capacity(17 + encrypted.len()); // 1 byte for method + 12 bytes for nonce + 4 bytes for length

        // Write the encryption method (1 byte)
        output.push(match self.method {
            EncryptionMethod::Aes => 0,
            EncryptionMethod::ChaCha => 1,
        });

        // Write the nonce (12 bytes)
        output.extend_from_slice(&nonce);

        // Write the encrypted data length (4 bytes)
        output.extend_from_slice(&(encrypted.len() as u32).to_le_bytes());

        // Write the encrypted data
        output.extend_from_slice(&encrypted);

        debug!(
            "Encoded {} bytes of data into {} bytes of octet-stream using {:?}",
            data.len(),
            output.len(),
            self.method
        );

        Ok(output)
    }

    fn decode(&self, content: &[u8]) -> Result<Vec<u8>> {
        if content.len() < 17 {
            // 1 + 12 + 4 bytes minimum
            return Err(RainbowError::InvalidData("Content too short".to_string()));
        }

        // Read the encryption method
        let method = match content[0] {
            0 => EncryptionMethod::Aes,
            1 => EncryptionMethod::ChaCha,
            _ => {
                return Err(RainbowError::InvalidData(
                    "Invalid encryption method".to_string(),
                ))
            }
        };

        // Read the nonce
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&content[1..13]);

        // Read the encrypted data length
        let mut len_bytes = [0u8; 4];
        len_bytes.copy_from_slice(&content[13..17]);
        let encrypted_len = u32::from_le_bytes(len_bytes) as usize;

        // Validate the length
        if content.len() < 17 + encrypted_len {
            return Err(RainbowError::InvalidData(
                "Content shorter than expected".to_string(),
            ));
        }

        // Extract the encrypted data
        let encrypted_data = &content[17..17 + encrypted_len];

        // Try to decrypt with both methods if the method doesn't match
        let result = if method == self.method {
            // Use the specified method
            match self.method {
                EncryptionMethod::Aes => self.decrypt_aes(encrypted_data, &nonce),
                EncryptionMethod::ChaCha => self.decrypt_chacha(encrypted_data, &nonce),
            }
        } else {
            // Try both methods
            self.decrypt_aes(encrypted_data, &nonce)
                .or_else(|_| self.decrypt_chacha(encrypted_data, &nonce))
        };

        match result {
            Ok(decrypted) => {
                debug!(
                    "Decoded {} bytes of data from {} bytes of octet-stream using {:?}",
                    decrypted.len(),
                    content.len(),
                    method
                );
                Ok(decrypted)
            }
            Err(e) => Err(e),
        }
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

    fn get_test_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        for i in 0..32 {
            key[i] = i as u8;
        }
        key
    }

    #[test]
    fn test_default() {
        let encoder = OctetEncoder::default();
        assert_eq!(encoder.method, EncryptionMethod::ChaCha);
    }

    #[test]
    fn test_random() {
        let encoder = OctetEncoder::random();
        assert!(matches!(
            encoder.method,
            EncryptionMethod::Aes | EncryptionMethod::ChaCha
        ));
    }

    #[test]
    fn test_new() {
        let key = get_test_key();
        let encoder = OctetEncoder::new(EncryptionMethod::Aes, key);
        assert_eq!(encoder.method, EncryptionMethod::Aes);
        assert_eq!(encoder.key, key);
    }

    #[test]
    fn test_encode_decode_aes() {
        init();
        let key = get_test_key();
        let encoder = OctetEncoder::new(EncryptionMethod::Aes, key);

        // 测试空数据
        let empty_data = b"";
        let encoded = encoder.encode(empty_data).unwrap();
        let decoded = encoder.decode(&encoded).unwrap();
        assert_eq!(decoded, empty_data);

        // 测试短数据
        let short_data = b"Hello, AES-GCM!";
        let encoded = encoder.encode(short_data).unwrap();
        let decoded = encoder.decode(&encoded).unwrap();
        assert_eq!(decoded, short_data);

        // 测试长数据
        let long_data: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
        let encoded = encoder.encode(&long_data).unwrap();
        let decoded = encoder.decode(&encoded).unwrap();
        assert_eq!(decoded, long_data);
    }

    #[test]
    fn test_encode_decode_chacha() {
        init();
        let key = get_test_key();
        let encoder = OctetEncoder::new(EncryptionMethod::ChaCha, key);

        // 测试空数据
        let empty_data = b"";
        let encoded = encoder.encode(empty_data).unwrap();
        let decoded = encoder.decode(&encoded).unwrap();
        assert_eq!(decoded, empty_data);

        // 测试短数据
        let short_data = b"Hello, ChaCha20-Poly1305!";
        let encoded = encoder.encode(short_data).unwrap();
        let decoded = encoder.decode(&encoded).unwrap();
        assert_eq!(decoded, short_data);

        // 测试长数据
        let long_data: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
        let encoded = encoder.encode(&long_data).unwrap();
        let decoded = encoder.decode(&encoded).unwrap();
        assert_eq!(decoded, long_data);
    }

    #[test]
    fn test_cross_method_decode() {
        init();
        let key = get_test_key();
        let test_data = b"Hello, Cross-Method!";

        // 使用相同密钥创建两个不同方法的编码器
        let aes_encoder = OctetEncoder::new(EncryptionMethod::Aes, key);
        let chacha_encoder = OctetEncoder::new(EncryptionMethod::ChaCha, key);

        // AES 编码，ChaCha 解码
        let encoded = aes_encoder.encode(test_data).unwrap();
        assert_eq!(encoded[0], 0); // 验证方法标记为 AES
        let decoded = chacha_encoder.decode(&encoded).unwrap();
        assert_eq!(decoded, test_data);

        // ChaCha 编码，AES 解码
        let encoded = chacha_encoder.encode(test_data).unwrap();
        assert_eq!(encoded[0], 1); // 验证方法标记为 ChaCha
        let decoded = aes_encoder.decode(&encoded).unwrap();
        assert_eq!(decoded, test_data);
    }

    #[test]
    fn test_wrong_key() {
        let key1 = get_test_key();
        let mut key2 = key1;
        key2[0] ^= 1; // 修改一个比特

        let encoder1 = OctetEncoder::new(EncryptionMethod::Aes, key1);
        let encoder2 = OctetEncoder::new(EncryptionMethod::Aes, key2);

        let test_data = b"Hello, Wrong Key!";
        let encoded = encoder1.encode(test_data).unwrap();
        assert!(encoder2.decode(&encoded).is_err());
    }

    #[test]
    fn test_invalid_data() {
        let encoder = OctetEncoder::default();

        // 测试空数据
        assert!(encoder.decode(&[]).is_err());

        // 测试太短的数据
        assert!(encoder.decode(&[1, 2, 3]).is_err());

        // 测试无效的方法
        let mut invalid_method = vec![2]; // 无效的方法字节
        invalid_method.extend_from_slice(&[0; 12]); // nonce
        invalid_method.extend_from_slice(&[0; 4]); // 长度
        invalid_method.extend_from_slice(&[0; 16]); // 最小数据大小
        assert!(encoder.decode(&invalid_method).is_err());

        // 测试数据被截断（在加密数据中间截断）
        let test_data = b"Test Data";
        let mut encoded = encoder.encode(test_data).unwrap();
        let len_start = 13; // 方法(1) + nonce(12)
        let mut len_bytes = [0u8; 4];
        len_bytes.copy_from_slice(&encoded[len_start..len_start + 4]);
        let encrypted_len = u32::from_le_bytes(len_bytes) as usize;
        let truncate_pos = len_start + 4 + encrypted_len / 2; // 截断一半加密数据
        encoded.truncate(truncate_pos);
        assert!(encoder.decode(&encoded).is_err());

        // 测试数据被修改（修改加密数据）
        let mut corrupted = encoder.encode(test_data).unwrap();
        let data_start = 17; // 方法(1) + nonce(12) + 长度(4)
        if corrupted.len() > data_start {
            corrupted[data_start] ^= 0xff; // 修改加密数据的第一个字节
            assert!(encoder.decode(&corrupted).is_err());
        }

        // 测试长度字段被修改
        let mut length_corrupted = encoder.encode(test_data).unwrap();
        let len_pos = 13; // 方法(1) + nonce(12)
        length_corrupted[len_pos] = 0xff; // 修改长度字段
        assert!(encoder.decode(&length_corrupted).is_err());
    }
}
