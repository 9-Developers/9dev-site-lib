//! Content-addressed URLs for static files.

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use sha2::{Digest, Sha256};
use std::{fs, io, path::Path};

/* ********************************************************************************************** *
 *                                           Constants                                            *
 * ********************************************************************************************** */

// Preserve URL path separators and RFC 3986 unreserved characters.
const PATH_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'/')
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

/* ********************************************************************************************** *
 *                                             Types                                              *
 * ********************************************************************************************** */

/// A content fingerprint and the root-relative URL that embeds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fingerprint {
    /// The first eight bytes of SHA-256 in lowercase hexadecimal, without ETag quotes.
    pub hash: String,
    /// The percent-encoded `/static/` URL, without a query or fragment.
    pub url: String,
}

/// A regular file under the caller's public directory.
pub struct FileAsset {
    /// Original UTF-8 filename relative to the root, with `/` separators.
    pub source: String,
    /// Asset fingerprint.
    pub fingerprint: Fingerprint,
}

/* ********************************************************************************************** *
 *                                        Public functions                                        *
 * ********************************************************************************************** */

/// Fingerprint the final bytes that the site will serve.
///
/// Supply a logical file path relative to the site's asset root:
/// UTF-8, separated by `/`, with no empty, `.` or `..` components.
/// Pass the original filename, not a URL.
/// Insert the hash before the filename's last extension, or append it if there is none.
/// A leading dot alone does not introduce an extension.
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
        encode_path(stem_path),
        encode_path(extension),
    );

    Fingerprint { hash, url }
}

/// Fingerprint a directory in source-path order.
/// Includes dotfiles; the site chooses which paths to publish.
/// Reject links and non-portable filenames rather than indexing a different file or leaving the
/// public directory.
pub fn walk(root: &Path) -> io::Result<Vec<FileAsset>> {
    let mut assets = Vec::new();

    visit(root, "", &mut assets)?;
    assets.sort_unstable_by(|a, b| a.source.cmp(&b.source));

    Ok(assets)
}

/* ********************************************************************************************** *
 *                                        Crate functions                                         *
 * ********************************************************************************************** */

pub(crate) fn encode_path(path: &str) -> percent_encoding::PercentEncode<'_> {
    utf8_percent_encode(path, PATH_ENCODE_SET)
}

/* ********************************************************************************************** *
 *                                       Private functions                                        *
 * ********************************************************************************************** */

fn visit(dir: &Path, prefix: &str, assets: &mut Vec<FileAsset>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().into_string().map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "asset filename must be UTF-8")
        })?;
        let source = format!("{prefix}{name}");

        if name.contains('\\') || name.chars().any(char::is_control) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid asset path: {source:?}"),
            ));
        }

        let kind = entry.file_type()?;
        if kind.is_dir() {
            visit(&entry.path(), &format!("{source}/"), assets)?;
        } else if kind.is_file() {
            let fingerprint = fingerprint(&source, &fs::read(entry.path())?);

            assets.push(FileAsset {
                source,
                fingerprint,
            });
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("not a regular asset: {source}"),
            ));
        }
    }

    Ok(())
}

/* ********************************************************************************************** *
 *                                             Tests                                              *
 * ********************************************************************************************** */

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::{fingerprint, walk};
    use std::path::Path;

    #[test]
    fn GIVEN_changed_bytes_WHEN_fingerprint_THEN_url_changes() {
        // When
        let empty = fingerprint("main.css", b"");
        let edited = fingerprint("main.css", b"abc");

        // Then
        assert_eq!(empty.hash, "e3b0c44298fc1c14");
        assert_eq!(empty.url, "/static/main.e3b0c44298fc1c14.css");

        assert_eq!(edited.hash, "ba7816bf8f01cfea");
        assert_eq!(edited.url, "/static/main.ba7816bf8f01cfea.css");
    }

    #[test]
    fn GIVEN_path_separator_WHEN_fingerprint_THEN_encodes_correctly() {
        // Given
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
            // Then
            assert_eq!(fingerprint(source, b"abc").url, url);
        }
    }

    #[test]
    fn GIVEN_source_WHEN_fingerprint_THEN_appends_fingerprint_correctly() {
        // Given
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
            // Then
            assert_eq!(fingerprint(source, b"abc").url, url);
        }
    }

    #[test]
    fn WHEN_walk_THEN_keeps_raw_sources_and_canonical_urls() {
        // Given
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/public");

        // When
        let files = walk(&root).unwrap();

        // Then
        let image = &files[2];
        let sources: Vec<_> = files.iter().map(|file| file.source.as_str()).collect();

        assert_eq!(
            sources,
            [
                ".well-known/example.txt",
                "images/.draft/notes.txt",
                "images/Two Words.svg"
            ]
        );
        assert_eq!(
            image.fingerprint,
            fingerprint(
                &image.source,
                include_bytes!("../tests/public/images/Two Words.svg")
            )
        );
        assert!(
            image
                .fingerprint
                .url
                .starts_with("/static/images/Two%20Words.")
        );
    }
}
