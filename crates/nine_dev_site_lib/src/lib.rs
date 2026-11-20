//! Shared browser assets for 9 Developers websites.

pub mod assets;
pub mod catalog;
#[cfg(feature = "axum")]
pub mod serving;
pub mod switcher;

/// Attribution and license text for the bundled browser assets, for use on a site's licenses page.
pub const THIRD_PARTY_NOTICES: &str = include_str!("../THIRD_PARTY_NOTICES.md");
