/*!
Utility functions for common operations used across the codebase.

This module provides various helper functions for:
- Random string generation
- HTTP header manipulation
- Other general purpose utilities

The utilities here are designed to be reusable components that simplify
common programming tasks throughout the application.
*/

use rand::{distributions::Alphanumeric, Rng};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::RainbowError;

pub struct HttpConstants {
    pub cookie_names: &'static [&'static str],
    pub post_paths: &'static [&'static str],
    pub get_paths: &'static [&'static str],
    pub error_details: &'static [(&'static str, &'static str)], // (状态码, 出现概率)
    pub status_codes: &'static [(u16, f32)],                    // (状态码, 出现概率)
}

pub const HTTP_CONSTANTS: HttpConstants = HttpConstants {
    cookie_names: &[
        "sessionId",
        "visitor",
        "track",
        "_ga",
        "_gid",
        "JSESSIONID",
        "cf_id",
    ],
    post_paths: &[
        "/api/v1/data",
        "/api/v1/upload",
        "/api/v2/submit",
        "/upload",
        "/submit",
        "/process",
    ],
    get_paths: &[
        "/",
        "/index.html",
        "/assets/main.css",
        "/js/app.js",
        "/images/logo.png",
        "/blog/latest",
        "/docs/guide",
    ],
    error_details: &[
        ("MIME_TYPE_MISSING", "Missing Content-Type header"),
        ("CONTENT_MISSING", "Missing content body"),
        ("BASE64_DECODE_FAILED", "Failed to decode Base64 data"),
        ("INVALID_PACKET_FORMAT", "Invalid packet format"),
        ("UNSUPPORTED_MIME_TYPE", "Unsupported MIME type"),
    ],
    status_codes: &[
        (200, 0.9), // 90% 概率
        (201, 0.025),
        (202, 0.025),
        (204, 0.025),
        (206, 0.025),
    ],
};

/// Generate random string with specified length
pub fn random_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// Generate realistic HTTP headers
pub fn generate_realistic_headers(is_request: bool) -> HeaderMap {
    let mut headers = HeaderMap::new();

    if is_request {
        headers.insert(
            "User-Agent",
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            ),
        );
        headers.insert(
            "Accept-Language",
            HeaderValue::from_static("en-US,en;q=0.9"),
        );
        headers.insert(
            "Accept-Encoding",
            HeaderValue::from_static("gzip, deflate, br"),
        );

        // 随机添加一些可选头部
        if rand::random::<bool>() {
            headers.insert("DNT", HeaderValue::from_static("1"));
        }
        if rand::random::<bool>() {
            headers.insert("Cache-Control", HeaderValue::from_static("max-age=0"));
        }
    } else {
        headers.insert("Server", HeaderValue::from_static("nginx/1.18.0"));
        headers.insert("X-Frame-Options", HeaderValue::from_static("SAMEORIGIN"));
        headers.insert(
            "X-Content-Type-Options",
            HeaderValue::from_static("nosniff"),
        );

        // 随机添加一些安全相关的头部
        if rand::random::<bool>() {
            headers.insert(
                "Strict-Transport-Security",
                HeaderValue::from_static("max-age=31536000; includeSubDomains"),
            );
        }
        if rand::random::<bool>() {
            headers.insert(
                "Content-Security-Policy",
                HeaderValue::from_static("default-src 'self'"),
            );
        }
    }

    headers
}

/// Generate random API path
pub fn generate_random_post_path() -> String {
    let api_paths = HTTP_CONSTANTS.post_paths;
    api_paths[rand::thread_rng().gen_range(0..api_paths.len())].to_string()
}

/// Generate random static resource path
pub fn generate_random_get_path() -> String {
    let static_paths = HTTP_CONSTANTS.get_paths;

    static_paths[rand::thread_rng().gen_range(0..static_paths.len())].to_string()
}

/// Check HTTP packet validity
pub fn validate_http_packet(packet: &[u8]) -> crate::Result<()> {
    if packet.len() < 16 {
        return Err(RainbowError::InvalidData("Packet too short".to_string()));
    }

    let content = String::from_utf8_lossy(packet);
    let first_line = content
        .lines()
        .next()
        .ok_or_else(|| RainbowError::InvalidData("Cannot get first line of packet".to_string()))?;

    if first_line.starts_with("HTTP/") && first_line.contains(" ") {
        return Ok(());
    }

    if first_line.split_whitespace().count() == 3
        && first_line.contains("HTTP/")
        && (first_line.starts_with("GET ") || first_line.starts_with("POST "))
    {
        return Ok(());
    }

    Err(RainbowError::InvalidData("Invalid HTTP format".to_string()))
}

/// Extract headers and content from HTTP packet
pub fn extract_http_parts(data: &[u8]) -> Option<(HeaderMap, Vec<u8>)> {
    let data_str = String::from_utf8_lossy(data);
    let mut parts = data_str.split("\r\n\r\n");

    let headers_str = parts.next()?;
    let content = parts.next()?.as_bytes().to_vec();

    let mut headers = HeaderMap::new();
    for line in headers_str.lines().skip(1) {
        // Skip request/response line
        if let Some((name, value)) = line.split_once(':') {
            if let Ok(header_name) = HeaderName::from_bytes(name.trim().as_bytes()) {
                if let Ok(header_value) = HeaderValue::from_str(value.trim()) {
                    headers.insert(header_name, header_value);
                }
            }
        }
    }

    Some((headers, content))
}
