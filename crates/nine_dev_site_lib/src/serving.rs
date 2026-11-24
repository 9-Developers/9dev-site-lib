//! Axum GET/HEAD response helpers for embedded assets.

use crate::{
    assets,
    catalog::{AliasCache, SiteCatalog},
};
use axum::{
    Router,
    http::{HeaderMap, HeaderValue, StatusCode, Uri, header},
    response::{IntoResponse, Response},
    routing::get,
};

pub const IMMUTABLE: &str = "public, max-age=31536000, immutable";

/// Mount `/static/{*path}` and one GET/HEAD route per stable alias. The site
/// supplies embedded bytes by source name; other paths keep the site's fallback.
pub fn asset_router<S: Clone + Send + Sync + 'static>(
    catalog: &'static SiteCatalog,
    bytes: fn(&str) -> Option<&'static [u8]>,
) -> Router<S> {
    let mut router = Router::new().route(
        "/static/{*path}",
        get(move |uri: Uri| async move {
            if let Some(asset) = assets::lookup(uri.path()) {
                return response(asset.bytes, asset.content_type, IMMUTABLE);
            }
            if let Some(entry) = catalog.immutable.by_url(uri.path())
                && let Some(file) = bytes(entry.source)
            {
                return response(file, content_type(entry.source), IMMUTABLE);
            }
            StatusCode::NOT_FOUND.into_response()
        }),
    );
    for alias in catalog.aliases {
        let entry = &alias.entry;
        router = router.route(
            entry.url,
            get(move |headers: HeaderMap| async move {
                let Some(file) = bytes(entry.source) else {
                    return StatusCode::NOT_FOUND.into_response();
                };
                let mime = content_type(entry.source);
                match alias.cache {
                    AliasCache::Revalidate => revalidate(file, mime, entry.hash, &headers),
                    AliasCache::Fixed(cache) => {
                        conditional(file, mime, entry.hash, &headers, cache)
                    }
                }
            }),
        );
    }
    router
}

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
    conditional(
        bytes,
        mime,
        hash,
        headers,
        "public, max-age=0, must-revalidate",
    )
}

fn conditional(
    bytes: &'static [u8],
    mime: &'static str,
    hash: &str,
    headers: &HeaderMap,
    cache: &'static str,
) -> Response {
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
        (StatusCode::NOT_MODIFIED, [(header::CACHE_CONTROL, cache)]).into_response()
    } else {
        response(bytes, mime, cache)
    };
    if let Ok(value) = HeaderValue::from_str(&tag) {
        response.headers_mut().insert(header::ETAG, value);
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{Alias, Catalog, Entry};
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn shared_asset_wins_when_the_site_registers_the_same_url() {
        use assets::theme::LATTE;

        static SITE: SiteCatalog = SiteCatalog {
            immutable: Catalog(&[Entry {
                source: "images/icons/latte.svg",
                url: LATTE.url,
                hash: "86af2ea9588a23fd",
            }]),
            aliases: &[],
        };
        let response = asset_router(&SITE, |_| Some(b"site copy"))
            .oneshot(Request::get(LATTE.url).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CACHE_CONTROL], IMMUTABLE);
        assert_eq!(
            to_bytes(response.into_body(), LATTE.bytes.len())
                .await
                .unwrap()
                .as_ref(),
            LATTE.bytes
        );
    }

    #[tokio::test]
    async fn encoded_immutable_and_discovery_urls_leave_other_paths_to_the_site() {
        const SOURCE: &str = ".well-known/Two Words.txt";
        const HASH: &str = "ba7816bf8f01cfea";
        const IMMUTABLE_URL: &str = "/static/.well-known/Two%20Words.ba7816bf8f01cfea.txt";
        const DISCOVERY: &str = "/.well-known/Two%20Words.txt";
        const HOUR: &str = "public, max-age=3600";
        static SITE: SiteCatalog = SiteCatalog {
            immutable: Catalog(&[Entry {
                source: SOURCE,
                url: IMMUTABLE_URL,
                hash: HASH,
            }]),
            aliases: &[Alias {
                entry: Entry {
                    source: SOURCE,
                    url: DISCOVERY,
                    hash: HASH,
                },
                cache: AliasCache::Fixed(HOUR),
            }],
        };
        let app = asset_router(&SITE, |source| (source == SOURCE).then_some(b"abc"))
            .fallback(|| async { (StatusCode::NOT_FOUND, "site 404") });
        assert_eq!(SITE.by_source(SOURCE).unwrap().url, IMMUTABLE_URL);
        for (path, status, cache, body) in [
            (IMMUTABLE_URL, 200, Some(IMMUTABLE), "abc"),
            (DISCOVERY, 200, Some(HOUR), "abc"),
            (
                "/static/.well-known/Two%20Words.0000000000000000.txt",
                404,
                None,
                "",
            ),
            (
                "/static/.well-known/Two%2520Words.ba7816bf8f01cfea.txt",
                404,
                None,
                "",
            ),
            ("/other-page", 404, None, "site 404"),
        ] {
            let response = app
                .clone()
                .oneshot(Request::get(path).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), status);
            assert_eq!(
                response
                    .headers()
                    .get(header::CACHE_CONTROL)
                    .map(|v| v.to_str().unwrap()),
                cache
            );
            assert_eq!(
                to_bytes(response.into_body(), 100).await.unwrap().as_ref(),
                body.as_bytes()
            );
        }
        for method in [axum::http::Method::GET, axum::http::Method::HEAD] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method(method)
                        .uri(DISCOVERY)
                        .header(header::IF_NONE_MATCH, format!("W/\"{HASH}\""))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), 304);
            assert_eq!(response.headers()[header::ETAG], format!("\"{HASH}\""));
            assert_eq!(response.headers()[header::CACHE_CONTROL], HOUR);
            assert!(to_bytes(response.into_body(), 0).await.unwrap().is_empty());
        }
        let head = app
            .oneshot(Request::head(DISCOVERY).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(head.status(), 200);
        assert_eq!(head.headers()[header::CACHE_CONTROL], HOUR);
        assert!(to_bytes(head.into_body(), 0).await.unwrap().is_empty());
    }
}
