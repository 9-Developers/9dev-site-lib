//! Declare which site files have public URLs, after generating the final bytes.

use crate::assets::{FileAsset, encode_path, walk};
use std::{collections::BTreeMap, fs, io, io::Write, path::Path};

/* ********************************************************************************************** *
 *                                             Types                                              *
 * ********************************************************************************************** */

/// Sources are relative to the public root.
///
/// Alias paths are absolute, unescaped URL paths:
/// e.g. `/Two Words.png` is served as `/Two%20Words.png`.
#[derive(Clone, Copy, Debug)]
pub enum Registration<'a> {
    /// One immutable, fingerprinted `/static/` URL.
    File(&'a str),
    /// Fingerprint a directory recursively, skipping hidden descendants.
    Directory(&'a str),
    /// Stable ETag-revalidated alias: (source, public path).
    Revalidate(&'a str, &'a str),
    /// Stable alias with `public, max-age=N`: (source, public path, seconds).
    MaxAge(&'a str, &'a str, u32),
}

/* ********************************************************************************************** *
 *                                        Public functions                                        *
 * ********************************************************************************************** */

/// Write a Rust expression of type `nine_dev_site_lib::catalog::SiteCatalog`.
///
/// Register a file and an alias for the same source to expose both URLs.
/// Unregistered files get no URLs.
/// Missing inputs and duplicate routes fail the build.
/// Write into `OUT_DIR` and watch `root` with Cargo's `rerun-if-changed`.
pub fn write(root: &Path, output: &Path, registrations: &[Registration<'_>]) -> io::Result<()> {
    fs::write(output, render(root, registrations)?)
}

/* ********************************************************************************************** *
 *                                       Private functions                                        *
 * ********************************************************************************************** */

#[inline]
fn entry(file: &FileAsset, url: &str) -> String {
    format!(
        "c::Entry {{ source: {:?}, url: {url:?}, hash: {:?} }}",
        file.source, file.fingerprint.hash
    )
}

#[inline]
fn invalid(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

#[inline]
fn relative_path(path: &str) -> bool {
    !path.contains('\\')
        && !path.chars().any(char::is_control)
        && path.split('/').all(|part| !matches!(part, "" | "." | ".."))
}

fn render(root: &Path, registrations: &[Registration<'_>]) -> io::Result<Vec<u8>> {
    use Registration::{Directory, File, MaxAge, Revalidate};

    let files = walk(root)?;
    let by_source: BTreeMap<_, _> = files
        .iter()
        .map(|file| (file.source.as_str(), file))
        .collect();
    let lookup = |source: &str| {
        by_source.get(source).copied().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, format!("missing asset: {source}"))
        })
    };
    let mut immutable: BTreeMap<String, String> = BTreeMap::new();
    let mut aliases: BTreeMap<String, (String, String)> = BTreeMap::new();
    let mut add_file = |file: &FileAsset| {
        let url = &file.fingerprint.url;

        if immutable.insert(url.clone(), entry(file, url)).is_some() {
            return Err(invalid(format!("duplicate asset URL: {url}")));
        }

        Ok(())
    };

    for &registration in registrations {
        match registration {
            File(source) => add_file(lookup(source)?)?,
            Directory(source) => {
                if !relative_path(source) || !root.join(source).is_dir() {
                    return Err(invalid(format!("invalid asset directory: {source}")));
                }

                let prefix = format!("{source}/");

                for file in &files {
                    if file.source.strip_prefix(&prefix).is_some_and(|rest| {
                        !rest.split('/').any(|component| component.starts_with('.'))
                    }) {
                        add_file(file)?;
                    }
                }
            }
            Revalidate(source, path) | MaxAge(source, path, _) => {
                let Some(relative) = path.strip_prefix('/').filter(|p| relative_path(p)) else {
                    return Err(invalid(format!("invalid alias path: {path}")));
                };

                if relative == "static" || relative.starts_with("static/") {
                    return Err(invalid(format!("alias in fingerprinted namespace: {path}")));
                }

                let url = format!("/{}", encode_path(relative));
                let cache = if let MaxAge(_, _, seconds) = registration {
                    format!("c::AliasCache::Fixed(\"public, max-age={seconds}\")")
                } else {
                    "c::AliasCache::Revalidate".to_owned()
                };
                let row = entry(lookup(source)?, &url);

                if aliases.insert(url, (row, cache)).is_some() {
                    return Err(invalid(format!("duplicate asset alias: {path}")));
                }
            }
        }
    }

    let mut rust = Vec::new();
    writeln!(rust, "{{ use ::nine_dev_site_lib::catalog as c;")?;
    writeln!(rust, "c::SiteCatalog {{ immutable: c::Catalog(&[")?;

    for row in immutable.values() {
        writeln!(rust, "{row},")?;
    }

    writeln!(rust, "]), aliases: &[")?;

    for (row, cache) in aliases.values() {
        writeln!(rust, "c::Alias {{ entry: {row}, cache: {cache} }},")?;
    }

    writeln!(rust, "] }} }}")?;

    Ok(rust)
}

/* ********************************************************************************************** *
 *                                             Tests                                              *
 * ********************************************************************************************** */

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::{Registration::*, *};

    #[test]
    fn GIVEN_directory_and_stable_alias_WHEN_render_THEN_can_publish_the_same_file()
    -> io::Result<()> {
        // Given
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/public");
        let image = "images/Two Words.svg";
        let canonical = crate::assets::fingerprint(image, &fs::read(root.join(image))?).url;
        let registrations = [Directory("images"), Revalidate(image, "/my logo.svg")];

        // When
        let rust = String::from_utf8(render(&root, &registrations)?).unwrap();

        // Then
        assert!(rust.contains(&canonical));
        assert!(rust.contains("/my%20logo.svg"));
        assert!(!rust.contains("notes.txt"), "published a hidden descendant");
        assert!(
            !rust.contains("example.txt"),
            "unregistered .well-known was published"
        );

        Ok(())
    }

    #[test]
    fn GIVEN_ambiguous_aliases_WHEN_render_THEN_returns_err() {
        // Given
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/public");
        let image = "images/Two Words.svg";

        for registrations in [
            vec![Revalidate(image, "/static/logo")],
            vec![Revalidate(image, "/images/../logo")],
        ] {
            // Then
            assert!(render(&root, &registrations).is_err(), "{registrations:?}");
        }
    }

    #[test]
    fn GIVEN_missing_inputs_WHEN_render_THEN_returns_err() {
        // Given
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/public");

        for registrations in [vec![File("missing.css")], vec![Directory("missing")]] {
            // Then
            assert!(render(&root, &registrations).is_err(), "{registrations:?}");
        }
    }

    #[test]
    fn GIVEN_overlapping_registrations_WHEN_render_THEN_returns_err() {
        // Given
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/public");
        let image = "images/Two Words.svg";

        for registrations in [
            vec![Directory("images"), File(image)],
            vec![
                Revalidate(image, "/logo"),
                MaxAge(".well-known/example.txt", "/logo", 60),
            ],
        ] {
            // Then
            assert!(render(&root, &registrations).is_err(), "{registrations:?}");
        }
    }
}
