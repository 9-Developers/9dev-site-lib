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

macro_rules! icons {
    ($($name:ident => $file:literal),+ $(,)?) => {
        $(
            pub static $name: super::Asset = super::Asset {
                url: env!(concat!("ICON_URL_", $file)),
                content_type: "image/svg+xml",
                bytes: include_bytes!(concat!("../assets/images/icons/", $file)),
            };
        )+
        pub(super) const ALL: &[&super::Asset] = &[$(&$name),+];
    };
}

/// Catppuccin color palettes and flavor icons. Append `#icon` only to icon URLs.
pub mod theme {
    /// Stylesheet defining `--ctp-*` colors for each `data-theme` flavor, with Mocha as the default.
    pub static PALETTE: super::Asset = super::Asset {
        url: env!("ASSET_URL_catppuccin.css"),
        content_type: "text/css",
        bytes: include_bytes!("../assets/catppuccin.css"),
    };

    icons! {
        LATTE => "latte.svg",
        FRAPPE => "frappe.svg",
        MACCHIATO => "macchiato.svg",
        MOCHA => "mocha.svg",
    }
}

/// Brand and interface icons. Append `#icon` to the URL for an SVG `<use>` element.
pub mod icons {
    icons! {
        ANGLE_DOWN => "angle-down.svg",
        ANILIST => "anilist.svg",
        BLUESKY => "bluesky.svg",
        DISCORD => "discord.svg",
        DISPLAY => "display.svg",
        DOWNLOAD => "download.svg",
        EMAIL => "email.svg",
        FACEBOOK => "facebook.svg",
        GITHUB => "github.svg",
        GIT_BRANCH => "git-branch.svg",
        GITLAB => "gitlab.svg",
        INSTAGRAM => "instagram.svg",
        LINKEDIN => "linkedin.svg",
        MASTODON => "mastodon.svg",
        SIGNAL => "signal.svg",
        SPOTIFY => "spotify.svg",
        THREADS => "threads.svg",
        TWITCH => "twitch.svg",
        WEBSITE => "website.svg",
        X => "x.svg",
        YOUTUBE => "youtube.svg",
    }
}

/// Vendored HTMX browser script, available with the `htmx` feature.
#[cfg(feature = "htmx")]
pub mod htmx {
    /// Load this script from `JS.url` in the site's template.
    pub static JS: super::Asset = super::Asset {
        url: env!("ASSET_URL_htmx.min.js"),
        content_type: "text/javascript",
        bytes: include_bytes!("../assets/htmx.min.js"),
    };
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
    icons::ALL
        .iter()
        .chain(theme::ALL)
        .copied()
        .chain([
            &theme::PALETTE,
            &crate::switcher::CSS,
            &crate::switcher::JS,
            #[cfg(feature = "htmx")]
            &htmx::JS,
        ])
        .find(|asset| asset.url == path)
}

#[cfg(test)]
mod tests {
    use super::{lookup, theme};
    use sha2::{Digest, Sha256};

    // Preserve the URL already used by Fulla and ixirsii for the upstream 4.0.0 bytes.
    const HTMX_URL: &str = "/static/htmx.min.e484d9171a9db30a.js";

    #[cfg(feature = "htmx")]
    #[test]
    fn htmx_serves_the_pinned_upstream_script_at_its_existing_url() {
        assert_eq!(super::htmx::JS.url, HTMX_URL);
        let asset = lookup(HTMX_URL).unwrap();
        assert_eq!(asset.content_type, "text/javascript");
        let digest = Sha256::digest(asset.bytes);
        let prefix = u64::from_be_bytes(digest[..8].try_into().unwrap());
        assert_eq!(asset.url, format!("/static/htmx.min.{prefix:016x}.js"));
        assert!(lookup("/static/htmx.min.js").is_none());
        assert!(lookup("/static/htmx.min.0000000000000000.js").is_none());
    }

    #[cfg(not(feature = "htmx"))]
    #[test]
    fn htmx_is_not_served_without_the_feature() {
        assert!(lookup(HTMX_URL).is_none());
    }

    #[test]
    fn every_packaged_svg_is_served_at_its_content_address() {
        use std::{fs, path::Path};

        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/images/icons");
        let mut count = 0;
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("svg") {
                continue;
            }
            let bytes = fs::read(&path).unwrap();
            let stem = path.file_stem().unwrap().to_str().unwrap();
            let prefix = u64::from_be_bytes(Sha256::digest(&bytes)[..8].try_into().unwrap());
            let url = format!("/static/images/icons/{stem}.{prefix:016x}.svg");
            let asset = lookup(&url).unwrap_or_else(|| panic!("unregistered icon: {url}"));
            assert_eq!(asset.bytes, bytes);
            assert_eq!(asset.content_type, "image/svg+xml");
            assert!(lookup(&format!("/static/images/icons/{stem}.svg")).is_none());
            count += 1;
        }
        assert_eq!(count, super::icons::ALL.len() + theme::ALL.len());
    }

    #[test]
    fn palette_is_served_only_at_its_content_address() {
        let palette = &theme::PALETTE;
        let hash = Sha256::digest(palette.bytes);
        let prefix = u64::from_be_bytes(hash[..8].try_into().unwrap());
        let url = format!("/static/catppuccin.{prefix:016x}.css");
        assert_eq!(palette.url, url);
        let served = lookup(&url).unwrap();
        assert_eq!(served.content_type, "text/css");
        assert_eq!(served.bytes, palette.bytes);
        assert!(lookup("/static/catppuccin.css").is_none());
        assert!(lookup("/static/catppuccin.0000000000000000.css").is_none());
    }

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
