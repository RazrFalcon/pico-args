use gumdrop::Options;

#[derive(Debug, Options)]
struct AppArgs {
    #[options(help = "Shows help")]
    help: bool,

    #[options(no_short, required, help = "Sets a number")]
    number: u32,

    #[options(no_short, help = "Sets an optional number")]
    opt_number: Option<u32>,

    #[options(no_short, help = "Sets width", default = "10", parse(try_from_str = "parse_width"))]
    width: u32,

    #[options(free, help = "Input file")]
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
    let args = AppArgs::parse_args_default_or_exit();
    println!("{:#?}", args);
}
