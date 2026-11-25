use nine_dev_site_lib::startup;
use std::{env, ffi::OsStr, fs, io, path::Path, process::Command};

// Run the actual startup sequence in a fresh, single-threaded process with
// its own working directory and environment (including the missing-file case).
fn probe(dir: &Path, vars: &[(&str, &OsStr)]) -> io::Result<String> {
    let mut cmd = Command::new(env::current_exe()?);
    cmd.current_dir(dir)
        .arg("probe")
        .env_remove("HOST")
        .env_remove("PORT")
        .env_remove("SITE_TEST_LABEL")
        .envs(vars.iter().copied());
    let output = cmd.output()?;
    assert!(output.status.success(), "{:?}", output.stderr);
    Ok(String::from_utf8(output.stdout).unwrap().trim().to_owned())
}

fn main() -> io::Result<()> {
    if env::args().nth(1).as_deref() == Some("probe") {
        match startup::load_local_dotenv().and_then(|()| {
            Ok((
                startup::listen_addr()?,
                startup::optional_env("SITE_TEST_LABEL")?,
            ))
        }) {
            Ok((address, label)) => println!("{address} {label:?}"),
            Err(err) => println!("{err}"),
        }
        return Ok(());
    }

    let root = tempfile::tempdir()?;
    fs::write(root.path().join(".env"), "PORT=9999\n")?;
    let dir = root.path().join("site");
    fs::create_dir(&dir)?;
    assert_eq!(probe(&dir, &[])?, "0.0.0.0:8080 None");
    for port in ["1", "65535"] {
        assert_eq!(
            probe(&dir, &[("PORT", port.as_ref())])?,
            format!("0.0.0.0:{port} None")
        );
    }
    for invalid in ["", "0", "bad", "65536"] {
        assert_eq!(
            probe(&dir, &[("PORT", invalid.as_ref())])?,
            "PORT must be an integer from 1 to 65535"
        );
    }
    assert_eq!(probe(&dir, &[("HOST", "::1".as_ref())])?, "[::1]:8080 None");
    for invalid in ["", "localhost", "[::1]", "127.0.0.1:8080"] {
        assert_eq!(
            probe(&dir, &[("HOST", invalid.as_ref())])?,
            "HOST must be an IP address"
        );
    }
    #[cfg(unix)]
    let non_unicode = {
        use std::os::unix::ffi::OsStrExt;
        let port = OsStr::from_bytes(b"\xff");
        assert_eq!(
            probe(&dir, &[("PORT", port)])?,
            "PORT must contain valid Unicode"
        );
        assert_eq!(
            probe(&dir, &[("HOST", port)])?,
            "HOST must contain valid Unicode"
        );
        port
    };

    let file = dir.join(".env");
    fs::write(
        &file,
        "\u{feff}PORT=8081\nHOST=127.0.0.1\nSITE_TEST_LABEL=\n",
    )?;
    assert_eq!(probe(&dir, &[])?, "127.0.0.1:8081 Some(\"\")");
    assert_eq!(
        probe(&dir, &[("HOST", "::1".as_ref()), ("PORT", "3000".as_ref())])?,
        "[::1]:3000 Some(\"\")"
    );
    #[cfg(unix)]
    assert_eq!(
        probe(&dir, &[("PORT", non_unicode)])?,
        "PORT must contain valid Unicode"
    );
    fs::write(
        &file,
        "PORT=8081\nSITE_TEST_LABEL=\"${PORT} \\\"\\\\\\$\\nnext\"\n",
    )?;
    assert_eq!(
        probe(&dir, &[])?,
        r#"0.0.0.0:8081 Some("8081 \"\\$\nnext")"#
    );

    for malformed in ["SECRET='private\n", "SECRET=private\0\n"] {
        fs::write(&file, malformed)?;
        assert_eq!(probe(&dir, &[])?, "could not load local .env file");
    }
    fs::remove_file(&file)?;
    fs::create_dir(&file)?;
    assert_eq!(probe(&dir, &[])?, "could not load local .env file");
    Ok(())
}
