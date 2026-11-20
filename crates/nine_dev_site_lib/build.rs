use std::{fs, io};

use nine_dev_site_build::assets::fingerprint;

fn main() -> io::Result<()> {
    const DIR: &str = "assets/images/icons";
    println!("cargo::rerun-if-changed={DIR}");
    for entry in fs::read_dir(DIR)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("svg") {
            continue;
        }
        let name = entry.file_name().into_string().map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "icon filename must be UTF-8")
        })?;
        let logical_path = format!("images/icons/{name}");
        let asset = fingerprint(&logical_path, &fs::read(path)?);
        println!("cargo::rustc-env=ICON_URL_{name}={}", asset.url);
    }
    for name in ["theme-switcher.css", "theme-switcher.js"] {
        let path = format!("assets/{name}");
        println!("cargo::rerun-if-changed={path}");
        let asset = fingerprint(name, &fs::read(path)?);
        println!("cargo::rustc-env=SWITCHER_URL_{name}={}", asset.url);
    }
    Ok(())
}
