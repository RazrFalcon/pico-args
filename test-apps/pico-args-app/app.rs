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

fn submain() -> Result<(), Box<std::error::Error>> {
    let mut args = Arguments::from_env();
    let args = AppArgs {
        help: args.contains(["-h", "--help"]),
        number: args.value_from_str("--number")?.unwrap_or(5),
        opt_number: args.value_from_str("--opt-number")?,
        width: args.value_from_fn("--width", parse_width)?.unwrap_or(10),
        free: args.free()?,
    };

    println!("{:#?}", args);
    Ok(())
}
