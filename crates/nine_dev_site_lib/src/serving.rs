//! Axum GET/HEAD response helpers for embedded assets.

use axum::{
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};

pub const IMMUTABLE: &str = "public, max-age=31536000, immutable";

pub fn content_type(source: &str) -> &'static str {
    mime_guess::from_path(source)
        .first_raw()
        .unwrap_or("application/octet-stream")
}

/// Serve bytes with a known media type and the site's chosen cache policy.
pub fn response(bytes: &'static [u8], mime: &'static str, cache: &'static str) -> Response {
    (
        [(header::CONTENT_TYPE, mime), (header::CACHE_CONTROL, cache)],
        bytes,
    )
        .into_response()
}

/// Serve a stable URL with a quoted build fingerprint. On an `If-None-Match`
/// hit, return an empty 304 carrying the same ETag and cache policy.
pub fn revalidate(
    bytes: &'static [u8],
    mime: &'static str,
    hash: &str,
    headers: &HeaderMap,
) -> Response {
    const CACHE: &str = "public, max-age=0, must-revalidate";
    let tag = format!("\"{hash}\"");
    let matches = headers.get_all(header::IF_NONE_MATCH).iter().any(|value| {
        let Ok(value) = value.to_str() else {
            return false;
        };
        let mut in_quotes = false;
        value.trim() == "*"
            || value
                .split(|c| {
                    if c == '"' {
                        in_quotes = !in_quotes;
                    }
                    c == ',' && !in_quotes
                })
                .any(|candidate| {
                    let candidate = candidate.trim();
                    candidate.strip_prefix("W/").unwrap_or(candidate) == tag
                })
    });
    let mut response = if matches {
        (StatusCode::NOT_MODIFIED, [(header::CACHE_CONTROL, CACHE)]).into_response()
    } else {
        response(bytes, mime, CACHE)
    };
    if let Ok(value) = HeaderValue::from_str(&tag) {
        response.headers_mut().insert(header::ETAG, value);
    }
    response
}
