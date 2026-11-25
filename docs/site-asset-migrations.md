# Site asset URL contracts

All three sites declare their public assets in `build.rs` with `catalog::write`
and mount them with `serving::asset_router`. Each generates font faces before
compiling Tailwind, so the stylesheet references the fingerprinted font bytes.

| Registration | HTTP behavior |
| --- | --- |
| `File` or `Directory` | Fingerprinted `/static/` URL; one year of immutable caching. An unknown or stale hash returns 404 without an asset cache header. |
| `Revalidate` | Exact stable URL; `public, max-age=0, must-revalidate` and a content ETag. |
| `MaxAge` | Exact stable URL; `public, max-age=N` and a content ETag. |

GET and HEAD on stable URLs honor `If-None-Match`: a match returns an empty
304 with the same ETag and cache policy. Paths outside `/static/` and the
registered aliases go to the site's own router.

## Site-specific URLs

- **fulla.fyi:** `/site.webmanifest`, `/favicon.ico`, `/icon-180.png`,
  `/icon-192.png`, and `/icon-512.png` are stable and revalidate. Fonts and
  site images use immutable URLs. No `/.well-known/` wildcard is mounted.
- **ixirsii.tech:** `/site.webmanifest` is stable. The 192px and 512px
  Android icons have both revalidating root URLs and immutable URLs.
  `/.well-known/security.txt` is the **only** URL for `security.txt`; its
  former fingerprinted URL returns 404. `/.well-known/bimi/logo.svg` also has
  an immutable URL. Both discovery paths have one-hour freshness and ETags.
  Dotfiles screenshots are indexed from `public/dotfiles/screenshots/`.
- **limpdick.io:** CSS uses immutable font URLs. Existing `/fonts/` URLs
  for the four fonts and `NOTICE.txt` still revalidate, as do the manifest
  and root icon URLs. Unknown pages use the styled page fallback. Telemetry
  labels `/robots.txt` as `static` even though it returns 404.
