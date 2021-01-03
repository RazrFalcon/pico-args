use std::path::PathBuf;
use std::ffi::OsStr;

#[derive(Debug)]
struct AppArgs {
    help: bool,
    number: u32,
    opt_number: Option<u32>,
    width: u32,
    input: Option<PathBuf>,
    output: Option<PathBuf>,
}

fn parse_width(s: &str) -> Result<u32, &'static str> {
    s.parse().map_err(|_| "not a number")
}

fn parse_path(s: &OsStr) -> Result<PathBuf, &'static str> {
    Ok(s.into())
}

fn main() {
    if let Err(e) = submain() {
        eprintln!("Error: {}.", e);
    }
}

fn submain() -> Result<(), pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();
    let args = AppArgs {
        // Checks that optional flag is present.
        help: pargs.contains(["-h", "--help"]),
        // Parses a required value that implements `FromStr`.
        // Returns an error if not present.
        number: pargs.value_from_str("--number")?,
        // Parses an optional value that implements `FromStr`.
        opt_number: pargs.opt_value_from_str("--opt-number")?,
        // Parses an optional value from `&str` using a specified function.
        width: pargs.opt_value_from_fn("--width", parse_width)?.unwrap_or(10),
        // Parses an optional value from `&OsStr` using a specified function.
        input: pargs.opt_value_from_os_str("--input", parse_path)?,
        // Parses a free-standing/positional argument.
        output: pargs.free_from_str()?,
    };

    // It's up to the caller what to do with the remaining arguments.
    let remaining = pargs.finish();
    if !remaining.is_empty() {
        eprintln!("Warning: unused arguments left: {:?}.", remaining);
    }

    println!("{:#?}", args);
    Ok(())
}
