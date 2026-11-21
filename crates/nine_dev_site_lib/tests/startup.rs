use nine_dev_site_lib::startup;
use std::{env, ffi::OsStr, fs, io, path::Path, process::Command};

// Run the actual startup sequence in a fresh, single-threaded process with
// its own working directory and environment (including the missing-file case).
fn probe(dir: &Path, port: Option<&OsStr>) -> io::Result<String> {
    let mut cmd = Command::new(env::current_exe()?);
    cmd.current_dir(dir)
        .arg("probe")
        .env_remove("PORT")
        .env_remove("SITE_TEST_LABEL");
    if let Some(value) = port {
        cmd.env("PORT", value);
    }
    let output = cmd.output()?;
    assert!(output.status.success(), "{:?}", output.stderr);
    Ok(String::from_utf8(output.stdout).unwrap().trim().to_owned())
}

fn main() -> io::Result<()> {
    if env::args().nth(1).as_deref() == Some("probe") {
        match startup::load_local_dotenv()
            .and_then(|()| Ok((startup::port()?, startup::optional_env("SITE_TEST_LABEL")?)))
        {
            Ok((port, label)) => println!("{port} {label:?}"),
            Err(err) => println!("{err}"),
        }
        return Ok(());
    }

    let root = tempfile::tempdir()?;
    fs::write(root.path().join(".env"), "PORT=9999\n")?;
    let dir = root.path().join("site");
    fs::create_dir(&dir)?;
    assert_eq!(probe(&dir, None)?, "3000 None");
    for port in ["1", "65535"] {
        assert_eq!(probe(&dir, Some(port.as_ref()))?, format!("{port} None"));
    }
    for invalid in ["", "0", "bad", "65536"] {
        assert_eq!(
            probe(&dir, Some(invalid.as_ref()))?,
            "PORT must be an integer from 1 to 65535"
        );
    }
    #[cfg(unix)]
    let non_unicode = {
        use std::os::unix::ffi::OsStrExt;
        let port = OsStr::from_bytes(b"\xff");
        assert_eq!(probe(&dir, Some(port))?, "PORT must contain valid Unicode");
        port
    };

    let file = dir.join(".env");
    fs::write(&file, "\u{feff}PORT=8081\nSITE_TEST_LABEL=\n")?;
    assert_eq!(probe(&dir, None)?, "8081 Some(\"\")");
    assert_eq!(probe(&dir, Some("8082".as_ref()))?, "8082 Some(\"\")");
    #[cfg(unix)]
    assert_eq!(
        probe(&dir, Some(non_unicode))?,
        "PORT must contain valid Unicode"
    );
    fs::write(
        &file,
        "PORT=8081\nSITE_TEST_LABEL=\"${PORT} \\\"\\\\\\$\\nnext\"\n",
    )?;
    assert_eq!(probe(&dir, None)?, r#"8081 Some("8081 \"\\$\nnext")"#);

    for malformed in ["SECRET='private\n", "SECRET=private\0\n"] {
        fs::write(&file, malformed)?;
        assert_eq!(probe(&dir, None)?, "could not load local .env file");
    }
    fs::remove_file(&file)?;
    fs::create_dir(&file)?;
    assert_eq!(probe(&dir, None)?, "could not load local .env file");
    Ok(())
}
