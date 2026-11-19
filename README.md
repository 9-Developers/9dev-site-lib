# 9 Developers Site Framework

[![Discord][badge-discord]][link-discord]
[![License][badge-bsd-3]][link-bsd-3]
[![Build][badge-workflow-build]][link-workflow-build]
[![Security][badge-workflow-security]][link-workflow-security]
[![Release][badge-releases]][link-releases]

A Cargo workspace for shared Rust utilities used by 9 Developers websites.
Its first member, [`nine_dev_site_build`](crates/nine_dev_site_build/), is for
site build scripts. A separate `nine_dev_site_lib` runtime crate is planned
for shared assets and components.

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
