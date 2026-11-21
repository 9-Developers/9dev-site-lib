# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- `nine_dev_site_lib::telemetry`: opt-in JSON logging, OTLP HTTP logs and metrics,
  and Axum request middleware. Includes a shared Grafana dashboard.
- `nine_dev_site_build`: `tailwind::write_catppuccin_aliases` provides Tailwind v4
  utilities for all 26 colors in the shared runtime palette.
### Changed

### Deprecated

### Fixed

### Removed

## nine_dev_site_lib - 0.5.0 (2026-11-20)

### Added

- Exact lookup for site-owned catalogs and opt-in `axum`
  responses for immutable files and stable files with ETag revalidation.

## nine_dev_site_build - 0.2.0 (2026-11-20)

### Added

- Standalone Tailwind compilation with write-only-if-changed CSS,
  and a deterministic directory walker returning source paths, hashes and URLs.

## nine_dev_site_lib - 0.4.0 (2026-11-20)

### Added

- Opt-in `htmx` feature: serve HTMX 4.0.0 at `assets::htmx::JS.url` through `assets::lookup`.

## nine_dev_site_lib - 0.3.0 (2026-11-20)

### Added

- `nine_dev_site_lib::assets::theme::PALETTE`: a fingerprinted stylesheet with all four Catppuccin flavors.

## nine_dev_site_lib - 0.2.0 (2026-11-20)

### Added

- Compact, keyboard-accessible theme switcher with a fingerprinted script and stylesheet.
- `nine_dev_site_lib::assets::icons`: 20 brand and interface SVG symbols,
  including Discord and GitHub, served through the existing asset lookup.
  artwork notices in `nine_dev_site_lib`.

## nine_dev_site_lib - 0.1.0 (2026-11-20)

### Added

- Runtime access to four embedded theme icons, their fingerprinted URLs, and

## nine_dev_site_build - 0.1.1 (2026-11-20)

### Added

- Crate license

## nine_dev_site_build - 0.1.0 (2026-11-19)

### Added

- Build-time asset fingerprinting with `nine_dev_site_build::assets::fingerprint`.
