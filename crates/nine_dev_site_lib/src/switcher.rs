//! Compact, server-rendered theme disclosure. Load `JS` before stylesheets.

use crate::assets::{Asset, icons, theme};
use askama::Template;

pub static CSS: Asset = Asset {
    url: env!("ASSET_URL_theme-switcher.css"),
    content_type: "text/css",
    bytes: include_bytes!("../assets/theme-switcher.css"),
};
pub static JS: Asset = Asset {
    url: env!("ASSET_URL_theme-switcher.js"),
    content_type: "text/javascript",
    bytes: include_bytes!("../assets/theme-switcher.js"),
};

/// Render directly in an Askama template with `{{ switcher|safe }}`.
/// Display labels are escaped by the inner template; stored theme keys are fixed.
#[derive(Template)]
#[template(path = "switcher.html")]
pub struct ThemeSwitcher<'a> {
    pub system: &'a str,
    pub latte: &'a str,
    pub frappe: &'a str,
    pub macchiato: &'a str,
    pub mocha: &'a str,
}

impl Default for ThemeSwitcher<'_> {
    fn default() -> Self {
        Self {
            system: "System",
            latte: "Latte (Light)",
            frappe: "Frappé",
            macchiato: "Macchiato",
            mocha: "Mocha (Dark)",
        }
    }
}

impl<'a> ThemeSwitcher<'a> {
    fn options(&self) -> [(&'static str, &'a str, &'static Asset); 5] {
        [
            ("system", self.system, &icons::DISPLAY),
            ("latte", self.latte, &theme::LATTE),
            ("frappe", self.frappe, &theme::FRAPPE),
            ("macchiato", self.macchiato, &theme::MACCHIATO),
            ("mocha", self.mocha, &theme::MOCHA),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_label_is_escaped_without_changing_the_option_key() {
        let html = ThemeSwitcher {
            latte: "<Light>",
            ..Default::default()
        }
        .render()
        .unwrap();
        assert!(html.contains("data-theme-value=\"latte\""));
        assert!(!html.contains("<Light>"));
        for asset in [
            &CSS,
            &JS,
            &icons::DISPLAY,
            &icons::ANGLE_DOWN,
            &theme::LATTE,
        ] {
            assert_eq!(crate::assets::lookup(asset.url).unwrap().bytes, asset.bytes);
        }
    }
}
