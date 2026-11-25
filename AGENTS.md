# Working in 9dev-site-lib

- Check `fulla.fyi`, `ixirsii.tech`, and `limpdick.io` before treating a site convention as shared. For now, use only `fulla.fyi` for downstream integration and testing.
- Adding an SVG under `crates/nine_dev_site_lib/assets/images/icons/` gives it a fingerprint. To expose and serve it, register it in the appropriate `icons!` list in `crates/nine_dev_site_lib/src/assets.rs`.
- For imported non-theme icons, `python3 scripts/import-icons.py --check` compares the vendored files with pinned upstream artwork; omit `--check` to regenerate. See `CONTRIBUTING.md` for tool prerequisites.
- Pass `uri.path()` directly to asset lookup. Shared assets need the `/static/` prefix, and site catalog URLs must stay percent-encoded.

## Verification

- Run both `cargo test --locked` and `cargo test --locked -p nine_dev_site_lib --all-features`. The default run skips optional runtime features.
- Match Forgejo linting with `cargo clippy --all-targets --all-features --locked -- -D warnings`.
- For startup or telemetry changes, run `cargo build --locked -p nine_dev_site_lib --features server --example telemetry_smoke` before `python3 scripts/check_otlp.py`. Install `opentelemetry-proto` for that Python interpreter. The script starts its own local OTLP receiver.
- Site CI fetches the smoke script by Git blob ID (`git hash-object scripts/check_otlp.py`). Land fixture updates here before updating downstream pins; a test-only change needs no crate release.

## Releases

- Bump only the crate being released, update the workspace `Cargo.lock`, and tag the merged commit `<crate-name>-vX.Y.Z`. If the runtime needs a new build-crate API, publish `nine_dev_site_build` first, then update the runtime’s `[build-dependencies]`.
