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
use fake::{faker::*, Fake};

use crate::stego::{Encoder, Random};
use crate::{RainbowError, Result};

#[derive(Debug, Clone)]

pub struct RssEncoder {
    feed_title: String,
    feed_link: String,
    feed_description: String,
    item_title: String,
    item_description: String,
}

impl Random for RssEncoder {
    fn random() -> Self {
        let company = company::en::CompanyName().fake::<String>();
        let domain = internet::en::DomainSuffix().fake::<String>();
        Self {
            feed_title: format!("{} News Feed", company),
            feed_link: format!(
                "https://news.{}.{}",
                company.to_lowercase().replace(" ", "-"),
                domain
            ),
            feed_description: format!("Latest updates from {}", company),
            item_title: format!(
                "{} {}",
                company::en::Industry().fake::<String>(),
                lorem::en::Word().fake::<String>()
            ),
            item_description: lorem::en::Paragraph(1..3).fake::<String>(),
        }
    }
}

impl Default for RssEncoder {
    fn default() -> Self {
        Self {
            feed_title: "Rainbow RSS Feed".to_string(),
            feed_link: "http://example.com/feed".to_string(),
            feed_description: "A steganographic RSS feed".to_string(),
            item_title: "Hidden Data".to_string(),
            item_description: "This item contains hidden data".to_string(),
        }
    }
}

impl Encoder for RssEncoder {
    fn name(&self) -> &'static str {
        "rss"
    }

    fn encode(&self, data: &[u8]) -> Result<Vec<u8>> {
        encode(
            data,
            &self.feed_title,
            &self.feed_link,
            &self.feed_description,
            &self.item_title,
            &self.item_description,
        )
    }

    fn decode(&self, content: &[u8]) -> Result<Vec<u8>> {
        decode(content)
    }

    fn get_mime_type(&self) -> &'static str {
        "application/xml"
    }
}

const RSS_TEMPLATE: &str = r#"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0">
<channel>
    <title>{title}</title>
    <link>{link}</link>
    <description>{description}</description>
    <language>en-us</language>
    <pubDate>{date}</pubDate>
    <lastBuildDate>{date}</lastBuildDate>
    <docs>http://blogs.law.harvard.edu/tech/rss</docs>
    <generator>Rainbow RSS Generator</generator>
    <item>
        <title>{item_title}</title>
        <link>{link}/item/1</link>
        <description>{item_description}</description>
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
pub fn encode(
    data: &[u8],
    feed_title: &str,
    feed_link: &str,
    feed_description: &str,
    item_title: &str,
    item_description: &str,
) -> Result<Vec<u8>> {
    let encoded = BASE64.encode(data);
    let date = get_rfc822_date();

    let rss = RSS_TEMPLATE
        .replace("{data}", &encoded)
        .replace("{date}", &date)
        .replace("{title}", feed_title)
        .replace("{link}", feed_link)
        .replace("{description}", feed_description)
        .replace("{item_title}", item_title)
        .replace("{item_description}", item_description);

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

    Err(RainbowError::InvalidData(
        "Failed to decode data".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rss() {
        let test_data = b"Hello, RSS Steganography!";
        let encoded = encode(
            test_data,
            "Test Feed",
            "http://test.com",
            "Test Feed Description",
            "Test Item",
            "Test Description",
        )
        .unwrap();
        assert!(!encoded.is_empty());
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, test_data);
    }

    #[test]
    fn test_empty_data() {
        let test_data = b"";
        let encoded = encode(
            test_data,
            "Test Feed",
            "http://test.com",
            "Test Feed Description",
            "Test Item",
            "Test Description",
        )
        .unwrap();
        assert!(!encoded.is_empty());
        let decoded = decode(&encoded).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_large_data() {
        let test_data: Vec<u8> = (0..2000).map(|i| (i % 256) as u8).collect();
        let encoded = encode(
            &test_data,
            "Test Feed",
            "http://test.com",
            "Test Feed Description",
            "Test Item",
            "Test Description",
        )
        .unwrap();
        let decoded = decode(&encoded).unwrap();
        assert!(!decoded.is_empty());
    }

    #[test]
    fn test_invalid_input() {
        let result = decode(b"");
        assert!(result.is_err());
        let result = decode(b"invalid content");
        assert!(result.is_err());
    }
}
