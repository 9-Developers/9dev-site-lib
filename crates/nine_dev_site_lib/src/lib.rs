//! Shared browser assets and optional server helpers for 9 Developers websites.

pub mod assets;
pub mod catalog;
pub mod icon_link;
#[cfg(feature = "axum")]
pub mod serving;
#[cfg(feature = "startup")]
pub mod startup;
pub mod switcher;
#[cfg(feature = "telemetry")]
pub mod telemetry;

/// Attribution and license text for the bundled browser assets, for use on a site's licenses page.
pub const THIRD_PARTY_NOTICES: &str = include_str!("../THIRD_PARTY_NOTICES.md");
