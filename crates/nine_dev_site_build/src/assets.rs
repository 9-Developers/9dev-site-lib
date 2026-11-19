//! Content-addressed URLs for static files.

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use sha2::{Digest, Sha256};

// Preserve URL path separators and RFC 3986 unreserved characters.
const PATH_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'/')
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

/// A content fingerprint and the root-relative URL that embeds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fingerprint {
    /// The first eight bytes of SHA-256 in lowercase hexadecimal, without ETag quotes.
    pub hash: String,
    /// The percent-encoded `/static/` URL, without a query or fragment.
    pub url: String,
}

/// Fingerprint the final bytes that the site will serve.
///
/// Supply a logical file path relative to the site's asset root: UTF-8, separated
/// by `/`, with no empty, `.` or `..` components. Pass the original filename,
/// not a URL. Insert the hash before the filename's last extension, or append
/// it if there is none. A leading dot alone does not introduce an extension.
///
/// ```
/// use nine_dev_site_build::assets::fingerprint;
///
/// let asset = fingerprint("images/icons/latte.svg", b"abc");
/// assert_eq!(asset.hash, "ba7816bf8f01cfea");
/// assert_eq!(asset.url, "/static/images/icons/latte.ba7816bf8f01cfea.svg");
/// ```
#[must_use]
pub fn fingerprint(logical_path: &str, bytes: &[u8]) -> Fingerprint {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut hash = String::with_capacity(16);
    for &byte in &digest[..8] {
        hash.push(char::from(HEX[usize::from(byte >> 4)]));
        hash.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }

    let name = logical_path.rsplit('/').next().unwrap_or(logical_path);
    let extension = match name.rfind('.') {
        Some(i) if i > 0 => &name[i..],
        _ => "",
    };
    let stem_path = &logical_path[..logical_path.len() - extension.len()];
    let url = format!(
        "/static/{}.{hash}{}",
        utf8_percent_encode(stem_path, PATH_ENCODE_SET),
        utf8_percent_encode(extension, PATH_ENCODE_SET),
    );
    Fingerprint { hash, url }
}

#[cfg(test)]
mod tests {
    use super::fingerprint;

    #[test]
    fn changing_the_served_bytes_changes_the_url() {
        let empty = fingerprint("main.css", b"");
        assert_eq!(empty.hash, "e3b0c44298fc1c14");
        assert_eq!(empty.url, "/static/main.e3b0c44298fc1c14.css");

        let edited = fingerprint("main.css", b"abc");
        assert_eq!(edited.hash, "ba7816bf8f01cfea");
        assert_eq!(edited.url, "/static/main.ba7816bf8f01cfea.css");
    }

    #[test]
    fn only_the_final_filename_receives_the_fingerprint() {
        for (source, url) in [
            (
                "vendor/htmx.min.js",
                "/static/vendor/htmx.min.ba7816bf8f01cfea.js",
            ),
            (
                "vendor/HTMX-LICENSE",
                "/static/vendor/HTMX-LICENSE.ba7816bf8f01cfea",
            ),
            (
                ".well-known/bimi/logo.svg",
                "/static/.well-known/bimi/logo.ba7816bf8f01cfea.svg",
            ),
            (
                "icons.v1/.hidden",
                "/static/icons.v1/.hidden.ba7816bf8f01cfea",
            ),
        ] {
            assert_eq!(fingerprint(source, b"abc").url, url);
        }
    }

    #[test]
    fn urls_encode_raw_filenames_without_losing_path_separators() {
        for (source, url) in [
            (
                "images/armors/Heavy Armor.png",
                "/static/images/armors/Heavy%20Armor.ba7816bf8f01cfea.png",
            ),
            (
                "icons/Frappé #1%.svg",
                "/static/icons/Frapp%C3%A9%20%231%25.ba7816bf8f01cfea.svg",
            ),
        ] {
            assert_eq!(fingerprint(source, b"abc").url, url);
        }
    }
}
