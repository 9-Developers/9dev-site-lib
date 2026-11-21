# 9 Developers Site Framework

[![Discord][badge-discord]][link-discord]
[![License][badge-bsd-3]][link-bsd-3]
[![Build][badge-workflow-build]][link-workflow-build]
[![Security][badge-workflow-security]][link-workflow-security]
[![Release][badge-releases]][link-releases]

A Cargo workspace for Rust websites. Use [`nine_dev_site_lib`](crates/nine_dev_site_lib/)
to serve shared icons, Catppuccin colors, a theme switcher, and optional HTMX.
Use [`nine_dev_site_build`](crates/nine_dev_site_build/) to fingerprint a site's
own assets at build time.

## Loading server configuration

Enable the `startup` feature of `nine_dev_site_lib` (0.6 or later). Load
`./.env` from synchronous `main`, before starting Tokio or other threads:

```rust
use nine_dev_site_lib::startup;

fn main() -> std::io::Result<()> {
    startup::load_local_dotenv()?;
    let port = startup::port()?;
    println!("Listening on port {port}");
    Ok(())
}
```

A missing file is fine; parent directories are never searched. Exported
variables take precedence. `port()` uses `3000` when `PORT` is absent and
rejects empty or out-of-range values. For other settings, `optional_env("HOST")?`
returns `None` for an absent variable and an error for a non-Unicode value.

## Using the shared icons

Add `nine_dev_site_lib` to the site's `[dependencies]`. Use `assets::icons` for
brand and interface icons and `assets::theme` for Latte, Frappé, Macchiato, and
Mocha. To link to an external site, build an `IconLink` with a site-owned URL,
an icon, and a nonempty label:

```rust
use nine_dev_site_lib::{assets::icons, icon_link::IconLink};

let source = IconLink::new("https://github.com/9-Developers", &icons::GITHUB, "9 Developers on GitHub")?;
let chat = IconLink::new("https://discord.gg/KTgshSbEUU", &icons::DISCORD, "Join the Discord community")?.icon_only();
```

Render `{{ source|safe }}` for a visible label, or `{{ chat|safe }}`
for an icon whose link is named by `aria-label`. Both open in a new tab. The
component escapes the label and hides the decorative SVG from assistive
technology. Style `.nd-icon-link` in your site CSS if needed.

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

## Serving HTMX

To serve HTMX 4.0.0 from your own site, enable the `htmx` feature:

```toml
[dependencies]
nine_dev_site_lib = { version = "0.5", features = ["htmx"] }
```

Reference the script's fingerprinted URL in your Askama template:

```html
<script src="{{ nine_dev_site_lib::assets::htmx::JS.url }}"></script>
```

The same `/static/` handler that calls `assets::lookup` serves the script.
Set any `htmx-config` options in your template before the script tag.

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

## Building a directory of assets

With `nine_dev_site_build = "0.2"` as a build dependency and the standalone
`tailwindcss` CLI on `PATH`, a site's `build.rs` can compile CSS and then
fingerprint everything under its public root:

```rust
use nine_dev_site_build::{assets, tailwind};
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for path in ["src/style", "templates", "public"] {
        println!("cargo:rerun-if-changed={path}");
    }
    let scratch = PathBuf::from(std::env::var("OUT_DIR")?).join("main.css");
    tailwind::compile(Path::new("src/style/main.css"), Path::new("public/main.css"), &scratch)?;
    for file in assets::walk(Path::new("public"))? {
        println!("cargo:rustc-env=ASSET_URL_{}={}", file.source, file.fingerprint.url);
    }
    Ok(())
}
```

`walk` returns entries in source-path order, including nested directories.
Use `file.fingerprint.hash` when generating an image index or an ETag.
`catalog::Catalog` in `nine_dev_site_lib` resolves site-defined entries by
source filename for templates or by exact encoded URL for requests.

Enable the runtime crate's optional `axum` feature to construct responses:
use `serving::response(bytes, mime, serving::IMMUTABLE)` for a fingerprinted
URL, or `serving::revalidate(bytes, mime, hash, &headers)` in a stable URL's
GET/HEAD handler. A matching `If-None-Match` returns `304 Not Modified`.

### Catppuccin utilities in Tailwind v4

With `nine_dev_site_build` 0.2.1 or later, add this before `tailwind::compile`
in `build.rs`:

```rust
tailwind::write_catppuccin_aliases(Path::new("src/style/catppuccin.aliases.css"))?;
```

Ignore that generated file in Git. In `src/style/main.css`, import it after
Tailwind and point class discovery at your site's templates:

```css
@import "tailwindcss" source(none);
@import "./catppuccin.aliases.css";
@source "../../templates";
```

This makes utilities like `bg-ctp-base` and `text-ctp-blue` available in
templates and in `@apply`. Add other `@source` directories if they contain
utility classes. Load the [palette](#catppuccin-palette) before your compiled
CSS so these colors follow the selected flavor.

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
