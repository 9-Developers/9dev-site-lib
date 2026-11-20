use std::{fs, io};

use nine_dev_site_build::assets::fingerprint;

fn main() -> io::Result<()> {
    for name in ["latte.svg", "frappe.svg", "macchiato.svg", "mocha.svg"] {
        let logical_path = format!("images/icons/{name}");
        let file = format!("assets/{logical_path}");
        println!("cargo::rerun-if-changed={file}");
        let asset = fingerprint(&logical_path, &fs::read(file)?);
        println!("cargo::rustc-env=ICON_URL_{name}={}", asset.url);
    }
    Ok(())
}
