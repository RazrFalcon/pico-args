use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct AppArgs {
    /// Sets a number.
    #[structopt(long = "number")]
    number: u32,

    /// Sets an optional number.
    #[structopt(long = "opt-number")]
    opt_number: Option<u32>,

    /// Sets width.
    #[structopt(long = "width", default_value = "10", parse(try_from_str = parse_width))]
    width: u32,

    #[structopt(name = "INPUT", parse(from_os_str))]
    input: std::path::PathBuf,
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
    let args = AppArgs::from_args();
    println!("{:#?}", args);
}
