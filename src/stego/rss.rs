/*!
 * RSS Feed Steganography Implementation
 *
 * This module implements steganography using RSS feed XML structure as the carrier.
 * The secret data is encoded into RSS feed items by:
 * - Converting secret data to base64
 * - Embedding data into RSS item descriptions and titles
 * - Preserving valid RSS feed structure
 *
 * Key features:
 * - Uses standard RSS 2.0 format
 * - Data is hidden in a way that produces valid RSS feeds
 * - Suitable for scenarios requiring covert communication via RSS
 */

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::prelude::*;

use crate::Result;

const RSS_TEMPLATE: &str = r#"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0">
<channel>
    <title>Rainbow RSS Feed</title>
    <link>http://example.com/feed</link>
    <description>A steganographic RSS feed</description>
    <language>en-us</language>
    <pubDate>{date}</pubDate>
    <lastBuildDate>{date}</lastBuildDate>
    <docs>http://blogs.law.harvard.edu/tech/rss</docs>
    <generator>Rainbow RSS Generator</generator>
    <item>
        <title>Hidden Data</title>
        <link>http://example.com/item/1</link>
        <description>This item contains hidden data</description>
        <pubDate>{date}</pubDate>
        <guid>{data}</guid>
    </item>
</channel>
</rss>"#;

/// Generate RFC822 format date string
fn get_rfc822_date() -> String {
    Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

/// Encode data into RSS XML
pub fn encode(data: &[u8]) -> Result<Vec<u8>> {
    let encoded = BASE64.encode(data);
    let date = get_rfc822_date();

    let rss = RSS_TEMPLATE
        .replace("{data}", &encoded)
        .replace("{date}", &date);

    Ok(rss.into_bytes())
}

/// Decode data from RSS XML
pub fn decode(data: &[u8]) -> Result<Vec<u8>> {
    let rss = String::from_utf8_lossy(data);

    // Find data in guid tag
    if let Some(start) = rss.find("<guid>") {
        if let Some(end) = rss[start..].find("</guid>") {
            let encoded = &rss[start + 6..start + end];
            if let Ok(decoded) = BASE64.decode(encoded) {
                return Ok(decoded);
            }
        }
    }

    Ok(Vec::new()) // Return empty vector if decoding fails
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rss() {
        let test_data = b"Hello, RSS Steganography!";
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
        let result = decode(b"invalid content").unwrap();
        assert!(result.is_empty());
    }
}
