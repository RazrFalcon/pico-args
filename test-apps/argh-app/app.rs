use argh::FromArgs;

/// App arguments.
#[derive(Debug, FromArgs)]
struct AppArgs {
    /// number
    #[argh(option)]
    number: u32,

    /// pptional number
    #[argh(option)]
    opt_number: Option<u32>,

    /// width
    #[argh(option, default = "10", from_str_fn(parse_width))]
    width: u32,

    /// free
    #[argh(positional)]
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
    let args: AppArgs = argh::from_env();
    println!("{:#?}", args);
}
