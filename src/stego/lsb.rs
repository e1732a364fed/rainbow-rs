/*!
 * mod lsb provides a [`LSBEncoder`] that can be used to encode and decode data using LSB steganography.
 *
 *
 */

use crate::{RainbowError, Result};
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};
use rand::seq::SliceRandom;
use std::{fs, path::PathBuf};

use super::{Encoder, Random};

fn create_random_image(width: u32, height: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = ImageBuffer::new(width, height);
    for pixel in img.pixels_mut() {
        *pixel = Rgba([
            rand::random::<u8>(),
            rand::random::<u8>(),
            rand::random::<u8>(),
            255,
        ]);
    }
    img
}

/// 默认 image_dir 为空。如果为空时调用 encode, 则会生成一个随机图片后再使用
#[derive(Debug, Clone)]
pub struct LSBEncoder {
    // LSB bits to use (1-8)
    lsb_bits: u8,
    // Directory containing cover images
    pub image_dir: Option<PathBuf>,
    // Current cover image
    cover_image: Option<DynamicImage>,
}

impl Default for LSBEncoder {
    /// 默认使用一张本包自带的 cc0图片作为 cover_image
    fn default() -> Self {
        let mut s = Self {
            lsb_bits: 1,
            image_dir: None,
            cover_image: None,
        };

        s.load_image_from_bytes(include_bytes!("../../res/Flower-436642-pixahive.jpg"))
            .unwrap();
        s
    }
}

impl Random for LSBEncoder {
    /// 随机生成一个 LSBEncoder. 使用 空 image_dir 和 随机图片作为 cover_image
    fn random() -> Self {
        Self {
            lsb_bits: rand::random::<u8>() % 3 + 1, // Use 1-3 LSB bits
            image_dir: None,
            cover_image: None,
        }
    }
}

impl LSBEncoder {
    /// It will load a random image from the image_dir and set it as self.cover_image
    pub fn new(image_dir: PathBuf) -> Result<Self> {
        let mut encoder = Self {
            lsb_bits: 1,
            image_dir: Some(image_dir.clone()),
            cover_image: None,
        };
        encoder.load_random_image_from_dir()?;
        Ok(encoder)
    }

    pub fn with_lsb_bits(lsb_bits: u8) -> Self {
        assert!(
            lsb_bits > 0 && lsb_bits <= 8,
            "LSB bits must be between 1 and 8"
        );
        Self {
            lsb_bits,
            image_dir: None,
            cover_image: None,
        }
    }

    /// 从指定路径加载图片作为 self.cover_image
    pub fn load_image(&mut self, image_path: PathBuf) -> Result<()> {
        let img = image::open(image_path)
            .map_err(|e| RainbowError::Other(format!("Failed to load image: {}", e)))?;
        self.cover_image = Some(img);
        Ok(())
    }

    pub fn load_image_from_bytes(&mut self, image_bytes: &[u8]) -> Result<()> {
        let img = image::load_from_memory(image_bytes)
            .map_err(|e| RainbowError::Other(format!("Failed to load image: {}", e)))?;
        self.cover_image = Some(img);
        Ok(())
    }

    /// 从 self.image_dir 中随机选择一张图片作为 self.cover_image
    pub fn load_random_image_from_dir(&mut self) -> Result<()> {
        if let Some(dir) = &self.image_dir {
            let entries: Vec<_> = fs::read_dir(dir)
                .map_err(|e| RainbowError::Other(format!("Failed to read directory: {}", e)))?
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let path = e.path();
                    path.extension()
                        .map(|ext| {
                            let ext = ext.to_string_lossy().to_lowercase();
                            ext == "png" || ext == "jpg" || ext == "jpeg"
                        })
                        .unwrap_or(false)
                })
                .collect();

            if entries.is_empty() {
                return Err(RainbowError::Other(
                    "No valid images found in directory".to_string(),
                ));
            }

            let entry = entries
                .choose(&mut rand::thread_rng())
                .ok_or_else(|| RainbowError::Other("Failed to choose random image".to_string()))?;

            self.load_image(entry.path())
        } else {
            Err(RainbowError::Other(
                "No image directory specified".to_string(),
            ))
        }
    }
}

impl Encoder for LSBEncoder {
    fn name(&self) -> &'static str {
        "lsb"
    }

    fn get_mime_type(&self) -> &'static str {
        "image/png"
    }

    fn encode(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Calculate required size
        let data_len = data.len();
        let bits_per_pixel = self.lsb_bits as usize * 3; // 3 channels (RGB)
        let total_bits = (data_len + 4) * 8; // 4 bytes for length + data bytes
        let pixels_needed = total_bits.div_ceil(bits_per_pixel); //(total_bits + bits_per_pixel - 1) / bits_per_pixel;

        // Add extra margin for safety, especially for multi-bit LSB
        let margin = if self.lsb_bits > 1 {
            // Add more margin for multi-bit LSB
            pixels_needed
        } else {
            100 // Original margin for 1-bit LSB
        };
        let min_pixels = pixels_needed + margin;

        // Get or create image
        let img = if let Some(cover) = &self.cover_image {
            let width = cover.width();
            let height = cover.height();
            if (width * height) as usize >= min_pixels {
                cover.to_rgba8()
            } else {
                return Err(RainbowError::Other(format!(
                    "Cover image too small to store data: need {} pixels, have {}",
                    min_pixels,
                    width * height
                )));
            }
        } else {
            // Make the image slightly larger than minimum required
            let width = (min_pixels as f64).sqrt().ceil() as u32;
            // Add a bit extra to height to ensure we have enough pixels
            let height = ((min_pixels as f64) / width as f64).ceil() as u32 + 2;
            create_random_image(width, height)
        };

        let mut img = img.clone();

        // Embed data length first (32 bits)
        let len_bytes = (data_len as u32).to_le_bytes();
        embed_bytes(self.lsb_bits, &mut img, &len_bytes, 0)?;

        // Embed actual data
        embed_bytes(self.lsb_bits, &mut img, data, 32)?;

        // Convert to PNG
        let mut buffer = Vec::new();
        let dynamic_image = DynamicImage::ImageRgba8(img);
        dynamic_image
            .write_with_encoder(image::codecs::png::PngEncoder::new(&mut buffer))
            .map_err(|e| RainbowError::Other(format!("Failed to save image: {}", e)))?;
        Ok(buffer)
    }

    fn decode(&self, content: &[u8]) -> Result<Vec<u8>> {
        let img = image::load_from_memory(content)
            .map_err(|e| RainbowError::Other(format!("Failed to load image: {}", e)))?;

        // Extract data length first (32 bits)
        let len_bytes = extract_bytes(self.lsb_bits, &img, 0, 4)?;
        let data_len = u32::from_le_bytes(len_bytes.try_into().unwrap()) as usize;

        // Extract actual data
        extract_bytes(self.lsb_bits, &img, 32, data_len)
    }
}

pub fn embed_bytes(
    lsb_bits: u8,
    img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    data: &[u8],
    start_bit: usize,
) -> Result<()> {
    let bits_per_pixel = lsb_bits as usize * 3;

    for (byte_idx, &byte) in data.iter().enumerate() {
        let start_pixel = (start_bit + byte_idx * 8) / bits_per_pixel;
        let bit_offset = (start_bit + byte_idx * 8) % bits_per_pixel;
        let channel_start = bit_offset / lsb_bits as usize;
        let bit_offset_in_channel = bit_offset % lsb_bits as usize;

        let x = (start_pixel as u32) % img.width();
        let y = (start_pixel as u32) / img.width();

        if y >= img.height() {
            return Err(RainbowError::Other(format!(
                "Image too small to store data: need y {}, have {}",
                y,
                img.height()
            )));
        }

        if lsb_bits == 1 {
            // Special handling for 1-bit LSB
            let mut bits_left = 8;
            let mut current_byte = byte;
            let mut current_pixel = start_pixel;
            let mut current_channel = channel_start;

            while bits_left > 0 {
                let x = (current_pixel as u32) % img.width();
                let y = (current_pixel as u32) / img.width();

                if y >= img.height() {
                    return Err(RainbowError::Other(format!(
                        "Image too small to store data: need y {}, have {}",
                        y,
                        img.height()
                    )));
                }

                let pixel = img.get_pixel_mut(x, y);

                while current_channel < 3 && bits_left > 0 {
                    // Clear LSB
                    pixel[current_channel] &= !1;
                    // Set LSB to current bit
                    pixel[current_channel] |= (current_byte >> 7) & 1;
                    current_byte = current_byte.wrapping_shl(1);
                    bits_left -= 1;
                    current_channel += 1;
                }

                if bits_left > 0 {
                    current_pixel += 1;
                    current_channel = 0;
                }
            }
        } else {
            let pixel = img.get_pixel_mut(x, y);
            let current_byte = byte;
            let mut bits_written = 0;
            let mut current_channel = channel_start;
            let mut current_offset = bit_offset_in_channel;

            while bits_written < 8 && current_channel < 3 {
                let available_bits = lsb_bits as usize - current_offset;
                let bits_to_write = std::cmp::min(available_bits, 8 - bits_written);
                let channel_mask = ((1 << bits_to_write) - 1) as u8;

                // Clear the target bits in the channel
                pixel[current_channel] &= !(channel_mask << current_offset);
                // Write the bits from the current byte
                let bits = ((current_byte >> (8 - bits_to_write - bits_written)) & channel_mask)
                    << current_offset;
                pixel[current_channel] |= bits;

                bits_written += bits_to_write;
                current_channel += 1;
                current_offset = 0;
            }

            // If we still have remaining bits, write them to the next pixel
            if bits_written < 8 {
                let next_pixel = start_pixel + 1;
                let x = (next_pixel as u32) % img.width();
                let y = (next_pixel as u32) / img.width();

                if y >= img.height() {
                    return Err(RainbowError::Other(format!(
                        "Image too small to store data: need y {}, have {}",
                        y,
                        img.height()
                    )));
                }

                let next_pixel = img.get_pixel_mut(x, y);
                current_channel = 0;

                while bits_written < 8 && current_channel < 3 {
                    let bits_to_write = std::cmp::min(lsb_bits as usize, 8 - bits_written);
                    let channel_mask = ((1 << bits_to_write) - 1) as u8;

                    // Clear the LSB bits in the channel
                    next_pixel[current_channel] &= !channel_mask;
                    // Write the bits from the current byte
                    let bits = (current_byte >> (8 - bits_to_write - bits_written)) & channel_mask;
                    next_pixel[current_channel] |= bits;

                    bits_written += bits_to_write;
                    current_channel += 1;
                }
            }
        }
    }

    Ok(())
}

pub fn extract_bytes(
    lsb_bits: u8,
    img: &DynamicImage,
    start_bit: usize,
    length: usize,
) -> Result<Vec<u8>> {
    let bits_per_pixel = lsb_bits as usize * 3;
    let mut result = Vec::with_capacity(length);

    for byte_idx in 0..length {
        let start_pixel = (start_bit + byte_idx * 8) / bits_per_pixel;
        let bit_offset = (start_bit + byte_idx * 8) % bits_per_pixel;
        let channel_start = bit_offset / lsb_bits as usize;
        let bit_offset_in_channel = bit_offset % lsb_bits as usize;

        let x = (start_pixel as u32) % img.width();
        let y = (start_pixel as u32) / img.width();

        if y >= img.height() {
            return Err(RainbowError::Other(format!(
                "Image too small to extract data: need y {}, have {}",
                y,
                img.height()
            )));
        }

        if lsb_bits == 1 {
            // Special handling for 1-bit LSB
            let mut byte = 0u8;
            let mut bits_read = 0;
            let mut current_pixel = start_pixel;
            let mut current_channel = channel_start;

            while bits_read < 8 {
                let x = (current_pixel as u32) % img.width();
                let y = (current_pixel as u32) / img.width();

                if y >= img.height() {
                    return Err(RainbowError::Other(format!(
                        "Image too small to extract data: need y {}, have {}",
                        y,
                        img.height()
                    )));
                }

                let pixel = img.get_pixel(x, y);

                while current_channel < 3 && bits_read < 8 {
                    // Get LSB and shift it to the right position
                    byte |= (pixel[current_channel] & 1) << (7 - bits_read);
                    bits_read += 1;
                    current_channel += 1;
                }

                if bits_read < 8 {
                    current_pixel += 1;
                    current_channel = 0;
                }
            }

            result.push(byte);
        } else {
            let pixel = img.get_pixel(x, y);
            let mut byte = 0u8;
            let mut bits_read = 0;
            let mut current_channel = channel_start;
            let mut current_offset = bit_offset_in_channel;

            while bits_read < 8 && current_channel < 3 {
                let available_bits = lsb_bits as usize - current_offset;
                let bits_to_read = std::cmp::min(available_bits, 8 - bits_read);
                let channel_mask = ((1 << bits_to_read) - 1) as u8;

                // Extract bits from the channel
                let channel_bits = (pixel[current_channel] >> current_offset) & channel_mask;
                byte |= channel_bits << (8 - bits_to_read - bits_read);

                bits_read += bits_to_read;
                current_channel += 1;
                current_offset = 0;
            }

            // If we need more bits, read from the next pixel
            if bits_read < 8 {
                let next_pixel = start_pixel + 1;
                let x = (next_pixel as u32) % img.width();
                let y = (next_pixel as u32) / img.width();

                if y >= img.height() {
                    return Err(RainbowError::Other(format!(
                        "Image too small to extract data: need y {}, have {}",
                        y,
                        img.height()
                    )));
                }

                let next_pixel = img.get_pixel(x, y);
                current_channel = 0;

                while bits_read < 8 && current_channel < 3 {
                    let bits_to_read = std::cmp::min(lsb_bits as usize, 8 - bits_read);
                    let channel_mask = ((1 << bits_to_read) - 1) as u8;

                    // Extract bits from the channel
                    let channel_bits = next_pixel[current_channel] & channel_mask;
                    byte |= channel_bits << (8 - bits_to_read - bits_read);

                    bits_read += bits_to_read;
                    current_channel += 1;
                }
            }

            result.push(byte);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::path::Path;

    fn setup_test_images() -> PathBuf {
        let test_dir = env::temp_dir().join("test_images");
        fs::create_dir_all(&test_dir).unwrap();

        // Create a test image with sufficient size
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(200, 200);
        let path = test_dir.join("test.png");
        img.save(&path).unwrap();

        test_dir
    }

    fn cleanup_test_images(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_image_steganography_with_cover() {
        let test_dir = setup_test_images();
        let encoder = LSBEncoder::new(test_dir.clone()).unwrap();
        let test_data = b"Hello, Image Steganography!";

        let encoded = encoder.encode(test_data).unwrap();
        let decoded = encoder.decode(&encoded).unwrap();

        assert_eq!(decoded, test_data);
        cleanup_test_images(&test_dir);
    }

    #[test]
    fn test_image_steganography_default() {
        let encoder = LSBEncoder::default();
        let test_data = b"Hello, Image Steganography!";

        let encoded = encoder.encode(test_data).unwrap();
        let decoded = encoder.decode(&encoded).unwrap();

        assert_eq!(decoded, test_data);
    }

    #[test]
    fn test_random_encoder() {
        let encoder = LSBEncoder::random();
        let test_data = b"Testing random encoder";

        let encoded = encoder.encode(test_data).unwrap();
        let decoded = encoder.decode(&encoded).unwrap();

        assert_eq!(decoded, test_data);
    }

    #[test]
    fn test_image_steganography_2bit() {
        let encoder = LSBEncoder::with_lsb_bits(2);
        let test_data = b"Testing 2-bit LSB steganography";

        let encoded = encoder.encode(test_data).unwrap();
        let decoded = encoder.decode(&encoded).unwrap();

        assert_eq!(decoded, test_data);
    }

    #[test]
    fn test_image_steganography_3bit() {
        let encoder = LSBEncoder::with_lsb_bits(3);
        let test_data = b"Testing 3-bit LSB steganography";

        let encoded = encoder.encode(test_data).unwrap();
        let decoded = encoder.decode(&encoded).unwrap();

        assert_eq!(decoded, test_data);
    }
}
