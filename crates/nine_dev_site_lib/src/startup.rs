//! Runtime configuration shared by the sites.

use std::{env, fs, io};

/// Load only `./.env`, preserving exported variables. Call from synchronous
/// `main` before starting worker threads. A missing file is fine.
///
/// # Errors
/// Reports an unreadable or malformed file without including its contents.
pub fn load_local_dotenv() -> io::Result<()> {
    const MESSAGE: &str = "could not load local .env file";
    let input = match fs::read_to_string(".env") {
        Ok(input) => input,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(io::Error::new(err.kind(), MESSAGE)),
    };
    // NUL cannot occur in an environment value. Reject it before dotenvy
    // calls set_var, whose panic message would contain the value.
    if input.contains('\0') {
        return Err(io::Error::new(io::ErrorKind::InvalidData, MESSAGE));
    }
    let input = input.strip_prefix('\u{feff}').unwrap_or(&input);
    for entry in dotenvy::from_read_iter(input.as_bytes()) {
        let (key, value) =
            entry.map_err(|_| io::Error::new(io::ErrorKind::InvalidData, MESSAGE))?;
        // dotenvy's bulk loader treats a non-Unicode export as absent. Skip
        // every existing OS value, even when the site never reads that key.
        if env::var_os(&key).is_some() {
            continue;
        }
        // The value is already expanded. Quote it literally for dotenvy's
        // setter so quotes, dollars and multiline values survive unchanged.
        let literal = value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('$', "\\$")
            .replace('\n', "\\n");
        dotenvy::from_read(format!("{key}=\"{literal}\"").as_bytes())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, MESSAGE))?;
    }
    Ok(())
}

/// Read a process variable, returning `None` only when it is absent.
///
/// # Errors
/// A non-Unicode value is reported by variable name only.
pub fn optional_env(name: &str) -> io::Result<Option<String>> {
    match env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{name} must contain valid Unicode"),
        )),
    }
}

/// Read the HTTP listen port, defaulting to 3000 when `PORT` is absent.
///
/// # Errors
/// An explicitly supplied port must be a Unicode integer in `1..=65535`.
pub fn port() -> io::Result<u16> {
    match optional_env("PORT")? {
        None => Ok(3000),
        Some(value) => value
            .parse::<std::num::NonZeroU16>()
            .map(std::num::NonZeroU16::get)
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "PORT must be an integer from 1 to 65535",
                )
            }),
    }
}
