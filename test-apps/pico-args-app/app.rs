use pico_args::Arguments;

#[derive(Debug)]
struct AppArgs {
    help: bool,
    number: u32,
    opt_number: Option<u32>,
    width: u32,
    free: Vec<String>,
}

fn parse_width(s: &str) -> Result<u32, String> {
    let w = s.parse().map_err(|_| "not a number")?;
    if w != 0 {
        Ok(w)
    } else {
        Err("width must be positive".to_string())
    }
}

fn main() {
    if let Err(e) = submain() {
        eprintln!("Error: {}.", e);
    }
}

fn submain() -> Result<(), pico_args::Error> {
    let mut pargs = Arguments::from_env();
    let args = AppArgs {
        help: pargs.contains(["-h", "--help"]),
        number: pargs.value_from_str("--number")?,
        opt_number: pargs.opt_value_from_str("--opt-number")?,
        width: pargs.opt_value_from_fn("--width", parse_width)?.unwrap_or(10),
        free: pargs.finish().iter().map(|s| s.to_str().unwrap().to_string()).collect(),
    };

    println!("{:#?}", args);
    Ok(())
}
