# 9 Developers Site Framework

[![Discord][badge-discord]][link-discord]
[![License][badge-bsd-3]][link-bsd-3]
[![Build][badge-workflow-build]][link-workflow-build]
[![Security][badge-workflow-security]][link-workflow-security]
[![Release][badge-releases]][link-releases]

A Cargo workspace for Rust websites. Use [`nine_dev_site_lib`](crates/nine_dev_site_lib/)
to serve shared icons, Catppuccin colors, and a theme switcher, and [`nine_dev_site_build`](crates/nine_dev_site_build/)
to fingerprint a site's own assets at build time.

## Using the shared icons

Add `nine_dev_site_lib` to the site's `[dependencies]`. Use `assets::icons` for
brand and interface icons and `assets::theme` for Latte, Frappé, Macchiato, and
Mocha. For example, pass `nine_dev_site_lib::assets::icons::GITHUB.url` as
`github_url` to a template:

```html
<a href="https://github.com/9-Developers" aria-label="9 Developers on GitHub">
  <svg aria-hidden="true" width="24" height="24"><use href="{{ github_url }}#icon"></use></svg>
</a>
```

In the site's `/static/` handler, pass the full request path to `assets::lookup`.
For example, inside an Axum handler returning `Response`:

```rust
use axum::{http::header, response::IntoResponse};
use nine_dev_site_lib::assets;

if let Some(icon) = assets::lookup(uri.path()) {
    return ([
        (header::CONTENT_TYPE, icon.content_type),
        (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
    ], icon.bytes).into_response();
}
// Continue with the site's own asset lookup.
```

Use `assets::icons::GIT_BRANCH` for a generic repository link. Include
`nine_dev_site_lib::THIRD_PARTY_NOTICES` on the site's
licenses page; brand marks remain subject to their owners' usage guidelines.

## Catppuccin palette

Link the palette before your site's stylesheet and serve it through `assets::lookup`:

```html
<link rel="stylesheet" href="{{ nine_dev_site_lib::assets::theme::PALETTE.url }}">
```

Use `var(--ctp-base)` for a background, `var(--ctp-text)` for text, or any of
the 26 `--ctp-*` colors. Set `data-theme` on `<html>` to `latte`, `frappe`,
`macchiato`, or `mocha`. A missing or unrecognized theme defaults to Mocha.
The palette works without JavaScript or Tailwind.

## Compact theme switcher

Place `{{ nine_dev_site_lib::switcher::ThemeSwitcher::default()|safe }}` in
your Askama header. The component escapes its labels before returning HTML.
To customize one, pass a `ThemeSwitcher { latte: "Yulia (Light)", ..Default::default() }`
instead. In the page head, load the script synchronously before stylesheets:

```html
<script src="{{ nine_dev_site_lib::switcher::JS.url }}"></script>
<link rel="stylesheet" href="{{ nine_dev_site_lib::switcher::CSS.url }}">
```

Serve both through `assets::lookup`, just like icons. The switcher sets
`html[data-theme]` for the shared palette or your own stylesheet. System follows
the OS and explicit choices persist under `localStorage.theme`. Set
`--nd-theme-surface` and `--nd-theme-text` to your panel colors if desired.

## Fingerprinting static assets

Declare `nine_dev_site_build` under `[build-dependencies]` in the site's
`Cargo.toml`. This keeps the fingerprinting code out of the server binary.
In `build.rs`, after compiling CSS or preparing other assets, pass the logical
path and the exact bytes to be embedded:

```rust
use nine_dev_site_build::assets::fingerprint;

let asset = fingerprint("images/icons/latte.svg", b"abc");
assert_eq!(asset.hash, "ba7816bf8f01cfea");
assert_eq!(asset.url, "/static/images/icons/latte.ba7816bf8f01cfea.svg");
```

Use the returned URL as-is in the site's asset table. For an SVG symbol,
append `#icon` in the template. The site chooses which files to embed, which
routes use immutable URLs, and which stable routes use the hash as an ETag.

[badge-discord]: https://img.shields.io/discord/452233740408717313?color=b4befe&label=discord&labelColor=1e1e2e&logo=discord&style=flat
[link-discord]: https://discord.gg/KTgshSbEUU
[badge-bsd-3]: https://img.shields.io/badge/License-BSD_3--Clause-89b4fa?labelColor=1e1e2e&logo=bsd&logoColor=white&style=flat
[link-bsd-3]: https://opensource.org/license/bsd-3-clause
[badge-workflow-build]: https://git.9dev.tech/9-Developers/9dev-site-lib/badges/workflows/build.yml/badge.svg?branch=main&event=push&label=Build&labelColor=1e1e2e&style=flat&logo=forgejo&logoColor=white
[link-workflow-build]: https://git.9dev.tech/9-Developers/9dev-site-lib/actions/workflows/build.yml
[badge-workflow-security]: https://git.9dev.tech/9-Developers/9dev-site-lib/badges/workflows/security.yml/badge.svg?branch=main&event=push&label=Security&labelColor=1e1e2e&style=flat&logo=trivy&logoColor=white
[link-workflow-security]: https://git.9dev.tech/9-Developers/9dev-site-lib/actions/workflows/security.yml
[badge-releases]: https://git.9dev.tech/9-Developers/9dev-site-lib/badges/release.svg?color=cba6f7&labelColor=1e1e2e&style=flat&logo=codeberg&logoColor=white
[link-releases]: https://git.9dev.tech/9-Developers/9dev-site-lib/releases
