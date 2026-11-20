//! Embed a named asset in a page and serve its bytes from the site's static route.

/// An embedded browser asset, ready to serve at its fingerprinted URL.
#[derive(Debug)]
pub struct Asset {
    /// Root-relative URL without a query or fragment.
    pub url: &'static str,
    /// Value for the HTTP `Content-Type` header.
    pub content_type: &'static str,
    /// The bytes used to compute the fingerprint.
    pub bytes: &'static [u8],
}

/// Catppuccin flavor icons. Append `#icon` to the URL for an SVG `<use>` element.
pub mod theme {
    use super::Asset;

    macro_rules! icon {
        ($file:literal) => {
            Asset {
                url: env!(concat!("ICON_URL_", $file)),
                content_type: "image/svg+xml",
                bytes: include_bytes!(concat!("../assets/images/icons/", $file)),
            }
        };
    }

    pub static LATTE: Asset = icon!("latte.svg");
    pub static FRAPPE: Asset = icon!("frappe.svg");
    pub static MACCHIATO: Asset = icon!("macchiato.svg");
    pub static MOCHA: Asset = icon!("mocha.svg");
}

/// Look up the exact, encoded request path, including `/static/`.
///
/// Pass `uri.path()` from the site's HTTP handler. Unknown and stale paths return `None`.
///
/// ```
/// use nine_dev_site_lib::assets::{lookup, theme};
///
/// let asset = lookup(theme::LATTE.url).unwrap();
/// assert_eq!(asset.content_type, "image/svg+xml");
/// assert!(lookup("/static/images/icons/latte.svg").is_none());
/// ```
#[must_use]
pub fn lookup(path: &str) -> Option<&'static Asset> {
    [
        &theme::LATTE,
        &theme::FRAPPE,
        &theme::MACCHIATO,
        &theme::MOCHA,
    ]
    .into_iter()
    .find(|asset| asset.url == path)
}

#[cfg(test)]
mod tests {
    use super::{lookup, theme};
    use sha2::{Digest, Sha256};

    #[test]
    fn serves_the_original_fulla_symbols_at_their_existing_urls() {
        // Fingerprints from Fulla before extraction: changing these URLs invalidates caches.
        for (icon, name, hash) in [
            (&theme::LATTE, "latte", "86af2ea9588a23fd"),
            (&theme::FRAPPE, "frappe", "0014b909d18c2099"),
            (&theme::MACCHIATO, "macchiato", "fa94051b39df5e2c"),
            (&theme::MOCHA, "mocha", "58d995f68329eec0"),
        ] {
            let url = format!("/static/images/icons/{name}.{hash}.svg");
            assert_eq!(icon.url, url);
            let served = lookup(&url).expect("registered URL");
            assert_eq!(served.content_type, "image/svg+xml");
            let digest = Sha256::digest(served.bytes);
            let prefix = u64::from_be_bytes(digest[..8].try_into().unwrap());
            assert_eq!(format!("{prefix:016x}"), hash);
            let svg = str::from_utf8(served.bytes).expect("UTF-8 SVG");
            assert!(svg.contains(r#"<symbol id="icon""#));
            assert!(svg.contains(r#"fill="currentColor""#));
        }
    }

    #[test]
    fn rejects_stale_unversioned_and_site_owned_paths() {
        for path in [
            "/static/images/icons/latte.0000000000000000.svg",
            "/static/images/icons/latte.svg",
            "/static/theme.js",
            "/static/images/icons/system.svg",
        ] {
            assert!(lookup(path).is_none(), "{path}");
        }
    }
}
