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

/// Catppuccin flavor icons. Append `#icon` to the URL for an SVG `<use>` element.
pub mod theme {
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
        .chain([&crate::switcher::CSS, &crate::switcher::JS])
        .find(|asset| asset.url == path)
}

#[cfg(test)]
mod tests {
    use super::{lookup, theme};
    use sha2::{Digest, Sha256};

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
