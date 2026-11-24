# Migrating the remaining site assets

Fulla is the first consumer. After the build helper and runtime changes are
released, migrate each other site independently:

- **ixirsii.tech:** replace the per-file `ASSET_URL_*` / `ASSET_HASH_*`
  variables and hand-maintained `STATIC_ASSETS`, `STABLE`, and discovery
  tables with `catalog::write` and `serving::asset_router`. Generate the four
  font faces with the build helper before compiling Tailwind. Preserve both
  stable Android icon aliases and their immutable URLs, the stable manifest,
  and both URLs for each explicitly published `.well-known` file (one-hour
  freshness on discovery paths). Update the dotfiles snapshot refresh script
  so adding screenshots no longer edits Rust catalog entries.
- **limpdick.io:** replace the generated image index, named tables, and root
  catch-all with `Directory("images")`, named registrations, and the shared
  asset router. Use fingerprinted font URLs in CSS but retain existing stable
  font, notice, icon, and manifest URLs with ETag revalidation. Preserve the
  theme preference migration. Revisit the catch-all telemetry override and
  its `/robots.txt` label when moving unmatched pages to the page fallback.

Before switching either router, record existing public URLs and compare them
against the generated catalog. Exercise GET and HEAD on immutable URLs,
encoded filenames, stable aliases and discovery documents. Check 304 headers,
stale hashes, page vs. static 404s, and the resulting bounded telemetry labels.
