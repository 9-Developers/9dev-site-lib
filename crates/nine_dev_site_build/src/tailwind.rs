//! Compile a site's Tailwind entrypoint with the standalone CLI on `PATH`.

use std::{fs, io, path::Path, process::Command};

/// A site-owned variable or static WOFF2 face. `source` is relative to the
/// public root; `style` and `weight` are trusted CSS descriptors from build.rs.
pub struct FontFace<'a> {
    pub family: &'a str,
    pub source: &'a str,
    pub style: &'a str,
    pub weight: &'a str,
}

/// Generate an importable stylesheet before compiling Tailwind. Register each
/// source as an immutable catalog asset so its fingerprinted URL resolves.
pub fn write_font_faces(root: &Path, destination: &Path, faces: &[FontFace<'_>]) -> io::Result<()> {
    use std::fmt::Write;
    let mut css = String::new();
    for face in faces {
        let url = crate::assets::fingerprint(face.source, &fs::read(root.join(face.source))?).url;
        let family = face.family.replace('\\', "\\\\").replace('\'', "\\'");
        writeln!(css, "@font-face {{ font-family: '{family}'; font-style: {}; font-weight: {}; font-display: swap; src: url(\"{url}\") format('woff2'); }}", face.style, face.weight)
            .expect("writing to a String cannot fail");
    }
    write_if_changed(destination, css.as_bytes())
}

/// Materialize the optional Tailwind v4 `@theme inline` bindings for the runtime
/// Catppuccin palette. Import this generated file from the site's CSS entrypoint.
/// The parent directory must exist. Identical files are left untouched.
pub fn write_catppuccin_aliases(destination: &Path) -> io::Result<()> {
    write_if_changed(
        destination,
        include_bytes!("../assets/catppuccin.aliases.css"),
    )
}

/// Write minified CSS only when the final bytes change, so a watched output
/// directory does not retrigger the build on every invocation. Pass a
/// separate scratch file in `OUT_DIR`: Tailwind's stdout mode appends a
/// newline that is not present when writing the same CSS to a file.
/// The caller owns Cargo's watches for templates, styles and other inputs.
pub fn compile(input: &Path, destination: &Path, scratch: &Path) -> io::Result<()> {
    let output = Command::new("tailwindcss")
        .arg("--input")
        .arg(input)
        .arg("--output")
        .arg(scratch)
        .arg("--minify")
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "tailwindcss failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    write_if_changed(destination, &fs::read(scratch)?)
}

fn write_if_changed(destination: &Path, bytes: &[u8]) -> io::Result<()> {
    match fs::read(destination) {
        Ok(existing) if existing == bytes => Ok(()),
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
        _ => fs::write(destination, bytes),
    }
}
