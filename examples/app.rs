use std::path::PathBuf;
use std::ffi::OsStr;

use pico_args::Arguments;

#[derive(Debug)]
struct AppArgs {
    help: bool,
    number: u32,
    opt_number: Option<u32>,
    width: u32,
    input: Option<PathBuf>,
    free: Vec<String>,
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
    let mut args = Arguments::from_env();
    let args = AppArgs {
        // Checks that optional flag is present.
        help: args.contains(["-h", "--help"]),
        // Parses a required value that implements `FromStr`.
        // Returns an error if not present.
        number: args.value_from_str("--number")?,
        // Parses an optional value that implements `FromStr`.
        opt_number: args.opt_value_from_str("--opt-number")?,
        // Parses an optional value from `&str` using a specified function.
        width: args.opt_value_from_fn("--width", parse_width)?.unwrap_or(10),
        // Parses an optional value from `&OsStr` using a specified function.
        input: args.opt_value_from_os_str("--input", parse_path)?,
        // Will return all free arguments or an error if any flags are left.
        free: args.free()?,
    };

    println!("{:#?}", args);
    Ok(())
}
