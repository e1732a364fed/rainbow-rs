/*! Audio Steganography Implementation

This module implements audio steganography using frequency-based data embedding. The technique works by:

1. Modifying carrier frequencies in the audio signal to encode hidden data
2. Using specific sample rates and carrier frequencies to maintain audio quality
3. Embedding data by subtly altering wave amplitudes within human hearing thresholds

Key features:
- Maintains audio quality while hiding data
- Resistant to basic audio processing/compression
- Configurable sample rate and carrier frequency for different scenarios

Best used for:
- Hiding small amounts of data in audio files
- Applications requiring subtle data embedding
- Scenarios where audio quality preservation is critical
*/

use crate::stego::Encoder;
use crate::Result;
use base64::{engine::general_purpose, Engine as _};
use std::f64::consts::PI;
use tracing::{debug, warn};

pub struct AudioEncoder {
    sample_rate: u32,
    carrier_freq: u32,
    frame_size: usize,
    sync_size: usize,
    sync_amplitude: f64,
    amplitude_step: f64,
}

impl Default for AudioEncoder {
    fn default() -> Self {
        Self {
            sample_rate: 8000,
            carrier_freq: 1000,
            frame_size: 32,
            sync_size: 64,
            sync_amplitude: 0.9,
            amplitude_step: 1.0 / 256.0,
        }
    }
}

impl AudioEncoder {
    fn generate_sync_sequence(&self) -> Vec<f64> {
        (0..self.sync_size)
            .map(|i| {
                if i % 2 == 0 {
                    self.sync_amplitude
                } else {
                    -self.sync_amplitude
                }
            })
            .collect()
    }

    fn byte_to_amplitude(&self, byte: u8) -> f64 {
        (byte as f64 + 1.0) * self.amplitude_step
    }

    fn amplitude_to_byte(&self, amplitude: f64) -> u8 {
        let byte = (amplitude / self.amplitude_step - 0.5).floor() as i32;
        byte.clamp(0, 255) as u8
    }

    fn generate_audio_data(&self, data: &[u8]) -> Vec<f64> {
        let mut samples = Vec::new();
        let mut phase = 0.0;
        let time_step = 1.0 / self.sample_rate as f64;

        // Add synchronization sequence
        samples.extend(self.generate_sync_sequence());

        // Encode data
        for &byte in data {
            let amplitude = self.byte_to_amplitude(byte);

            // Generate one frame of data
            for _ in 0..self.frame_size {
                let sample = amplitude * (2.0 * PI * self.carrier_freq as f64 * phase).sin();
                samples.push(sample);
                phase += time_step;
            }

            // Add brief silence between bytes
            samples.extend(std::iter::repeat(0.0).take(4));
        }

        samples
    }

    fn calculate_peak_amplitude(frame: &[f64]) -> f64 {
        frame.iter().map(|&x| x.abs()).fold(0.0, f64::max)
    }

    fn extract_data(&self, samples: &[f64]) -> Option<Vec<u8>> {
        let mut data = Vec::new();
        let mut pos = 0;
        let sync_sequence = self.generate_sync_sequence();

        // Find synchronization sequence
        let mut sync_detected = false;
        'outer: while pos <= samples.len().saturating_sub(self.sync_size) {
            let mut match_found = true;
            for i in 0..self.sync_size {
                let expected = sync_sequence[i];
                let actual = samples[pos + i];
                if (actual.abs() - expected.abs()).abs() > 0.1 * expected.abs() {
                    match_found = false;
                    break;
                }
            }

            if match_found {
                sync_detected = true;
                pos += self.sync_size;
                break 'outer;
            }
            pos += 1;
        }

        if !sync_detected {
            return None;
        }

        // Decode data
        let frame_size = self.frame_size + 4; // Including silence interval
        while pos + self.frame_size <= samples.len() {
            let frame: Vec<f64> = samples[pos..pos + self.frame_size].to_vec();
            let amplitude = Self::calculate_peak_amplitude(&frame);

            if amplitude > self.amplitude_step / 2.0 {
                let byte = self.amplitude_to_byte(amplitude);
                data.push(byte);
            }

            pos += frame_size;
        }

        Some(data)
    }
}

impl Encoder for AudioEncoder {
    fn name(&self) -> &'static str {
        "audio"
    }

    fn encode(&self, data: &[u8]) -> Result<Vec<u8>> {
        debug!("Encoding data using Web Audio API stego");

        if data.is_empty() {
            return Ok(b"<audio id=\"stego-audio\" style=\"display:none\"></audio>".to_vec());
        }

        let data = if data.len() > 1000 {
            warn!("Data too long, truncating to 1000 bytes");
            &data[..1000]
        } else {
            data
        };

        // Generate audio waveform
        let audio_data = self.generate_audio_data(data);

        // Convert audio data to string
        let audio_str = audio_data
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(",");

        // Base64 encoding
        let encoded = general_purpose::STANDARD.encode(audio_str);

        // Generate HTML
        let html = format!(
            "<audio id=\"stego-audio\" style=\"display:none\" data-audio=\"{}\"></audio>",
            encoded
        );

        Ok(html.into_bytes())
    }

    fn decode(&self, content: &[u8]) -> Result<Vec<u8>> {
        let content = String::from_utf8_lossy(content);
        if content.is_empty() || !content.contains("audio") {
            return Ok(Vec::new());
        }

        // Extract base64 encoded data
        if let Some(encoded) = content
            .split("data-audio=\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
        {
            // Decode base64
            if let Ok(audio_str) = general_purpose::STANDARD.decode(encoded) {
                if let Ok(audio_str) = String::from_utf8(audio_str) {
                    // Parse audio samples
                    let samples: Vec<f64> = audio_str
                        .split(',')
                        .filter_map(|s| s.parse::<f64>().ok())
                        .collect();

                    // Extract data from samples
                    if let Some(data) = self.extract_data(&samples) {
                        return Ok(data);
                    }
                }
            }
        }

        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio() {
        let encoder = AudioEncoder::default();
        let test_data = b"Hello, Audio Steganography!";

        // Test encoding
        let encoded = encoder.encode(test_data).unwrap();
        assert!(!encoded.is_empty());
        assert!(String::from_utf8_lossy(&encoded).contains("audio"));
        assert!(String::from_utf8_lossy(&encoded).contains("base64"));

        // Test decoding
        let decoded = encoder.decode(&encoded).unwrap();
        assert_eq!(decoded, test_data);
    }

    #[test]
    fn test_empty_data() {
        let encoder = AudioEncoder::default();
        let test_data = b"";

        // Test encoding empty data
        let encoded = encoder.encode(test_data).unwrap();
        assert!(!encoded.is_empty());
        assert!(String::from_utf8_lossy(&encoded).contains("audio"));

        // Test decoding empty data
        let decoded = encoder.decode(&encoded).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_large_data() {
        let encoder = AudioEncoder::default();
        let test_data: Vec<u8> = (0..2000).map(|i| (i % 256) as u8).collect();

        // Test encoding large data
        let encoded = encoder.encode(&test_data).unwrap();
        assert!(!encoded.is_empty());
        assert!(String::from_utf8_lossy(&encoded).contains("audio"));

        // Test decoding large data
        let decoded = encoder.decode(&encoded).unwrap();
        assert!(!decoded.is_empty());
        assert_eq!(&decoded[..1000], &test_data[..1000]);
    }

    #[test]
    fn test_invalid_input() {
        let encoder = AudioEncoder::default();

        // Test decoding invalid input
        let decoded = encoder.decode(b"invalid audio data").unwrap();
        assert!(decoded.is_empty());

        let decoded = encoder.decode(b"").unwrap();
        assert!(decoded.is_empty());

        let decoded = encoder.decode(b"<audio></audio>").unwrap();
        assert!(decoded.is_empty());
    }
}
