//! Compile a site's Tailwind entrypoint with the standalone CLI on `PATH`.

use std::{fs, io, path::Path, process::Command};

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
    let css = fs::read(scratch)?;
    match fs::read(destination) {
        Ok(existing) if existing == css => Ok(()),
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
        _ => fs::write(destination, css),
    }
}
