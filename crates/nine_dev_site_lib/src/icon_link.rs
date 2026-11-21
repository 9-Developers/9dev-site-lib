//! Accessible external links using fingerprinted SVG symbols.

use crate::assets::Asset;
use askama::Template;
use std::fmt;

/// A link with a visible label by default. Use [`Self::icon_only`] for compact navigation.
#[derive(Template)]
#[template(path = "icon_link.html")]
pub struct IconLink<'a> {
    href: &'static str,
    icon: &'static Asset,
    label: &'a str,
    icon_only: bool,
}

impl<'a> IconLink<'a> {
    /// Link to a site-authored destination using a symbol from [`crate::assets::icons`].
    ///
    /// # Errors
    /// Returns [`EmptyLabel`] if the label is empty or contains only whitespace.
    pub fn new(
        href: &'static str,
        icon: &'static Asset,
        label: &'a str,
    ) -> Result<Self, EmptyLabel> {
        let label = label.trim();
        if label.is_empty() {
            return Err(EmptyLabel);
        }
        Ok(Self {
            href,
            icon,
            label,
            icon_only: false,
        })
    }

    /// Put the label in `aria-label` instead of displaying it alongside the icon.
    #[must_use]
    pub const fn icon_only(mut self) -> Self {
        self.icon_only = true;
        self
    }
}

/// The link would have no accessible name.
#[derive(Debug)]
pub struct EmptyLabel;

impl fmt::Display for EmptyLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("an icon link needs a nonempty label")
    }
}

impl std::error::Error for EmptyLabel {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::icons;

    #[test]
    fn both_presentations_keep_one_escaped_name_and_a_decorative_icon() {
        let link = IconLink::new(
            "https://example.com/?a=1&b=2",
            &icons::GITHUB,
            "<Source> \"repo\"",
        )
        .unwrap();
        let visible = link.render().unwrap();
        let compact = link.icon_only().render().unwrap();
        for html in [&visible, &compact] {
            assert!(html.contains(&format!("{}#icon", icons::GITHUB.url)));
            assert!(html.contains("aria-hidden=\"true\""));
            assert!(html.contains("focusable=\"false\""));
            assert!(!html.contains("<Source>"));
            assert!(!html.contains("?a=1&b=2"));
        }
        assert!(visible.contains("<span>"));
        assert!(!visible.contains("aria-label="));
        let name = visible
            .split_once("<span>")
            .unwrap()
            .1
            .split_once("</span>")
            .unwrap()
            .0;
        assert!(name.contains("Source") && name.contains("repo"));
        assert!(compact.contains(&format!("aria-label=\"{name}\"")));
        assert!(!compact.contains("<span>"));
    }

    #[test]
    fn whitespace_alone_cannot_name_a_link() {
        assert!(IconLink::new("https://example.com", &icons::DISCORD, " \t\u{a0}").is_err());
    }
}
