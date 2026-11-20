//! Lookup of site-owned assets. The site declares the entries and embeds the bytes.

/// One build-time mapping. `url` is the exact encoded request path, including
/// its leading slash. Stable URLs carry the same content hash as immutable ones.
pub struct Entry {
    pub source: &'static str,
    pub url: &'static str,
    pub hash: &'static str,
}

pub struct Catalog(pub &'static [Entry]);

impl Catalog {
    pub fn by_source(&self, source: &str) -> Option<&'static Entry> {
        self.0.iter().find(|entry| entry.source == source)
    }

    /// Pass `uri.path()` without percent-decoding it.
    pub fn by_url(&self, path: &str) -> Option<&'static Entry> {
        self.0.iter().find(|entry| entry.url == path)
    }
}
