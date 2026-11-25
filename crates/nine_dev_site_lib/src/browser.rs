//! Browser response headers for same-origin sites.

use axum::{
    http::{HeaderValue, header},
    response::Response,
};

/// Apply the shared CSP, framing and MIME-sniffing policy to a response.
/// Use with [`axum::middleware::map_response`] on the completed router.
/// An existing `Referrer-Policy` takes precedence; cache headers are untouched.
pub async fn security_headers(mut response: Response) -> Response {
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; object-src 'none'; base-uri 'none'; \
             frame-ancestors 'none'; form-action 'self'",
        ),
    );
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers
        .entry(header::REFERRER_POLICY)
        .or_insert(HeaderValue::from_static("strict-origin-when-cross-origin"));
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn public_response_gets_the_full_baseline_without_a_cache_policy() {
        let response = security_headers(().into_response()).await;
        let headers = response.headers();
        assert_eq!(
            headers[header::CONTENT_SECURITY_POLICY],
            "default-src 'self'; object-src 'none'; base-uri 'none'; \
             frame-ancestors 'none'; form-action 'self'"
        );
        assert_eq!(headers[header::X_FRAME_OPTIONS], "DENY");
        assert_eq!(headers[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
        assert_eq!(
            headers[header::REFERRER_POLICY],
            "strict-origin-when-cross-origin"
        );
        assert!(!headers.contains_key(header::CACHE_CONTROL));
    }

    #[tokio::test]
    async fn preserves_route_referrer_and_cache_policies() {
        let response = [
            (header::REFERRER_POLICY, "same-origin"),
            (header::CACHE_CONTROL, "no-store"),
        ]
        .into_response();
        let response = security_headers(response).await;
        assert_eq!(response.headers()[header::REFERRER_POLICY], "same-origin");
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    }
}
