use clap::{Arg, App, value_t};

#[derive(Debug)]
struct AppArgs {
    help: bool,
    number: u32,
    opt_number: Option<u32>,
    width: u32,
    free: Vec<String>,
}

fn is_width(s: String) -> Result<(), String> {
    let w: u32 = s.parse().map_err(|_| "not a number")?;
    if w != 0 {
        Ok(())
    } else {
        Err("width must be positive".to_string())
    }
}

fn main() {
    let matches = App::new("App")
        .arg(Arg::with_name("help")
            .short("h")
            .long("help"))
        .arg(Arg::with_name("number")
            .long("number")
            .default_value("5")
            .takes_value(true))
        .arg(Arg::with_name("opt-number")
            .long("opt-number")
            .takes_value(true))
        .arg(Arg::with_name("width")
            .long("width")
            .default_value("10")
            .validator(is_width)
            .takes_value(true))
        .arg(Arg::with_name("input")
            .index(1))
        .get_matches();

    let mut args = AppArgs {
        help: matches.is_present("help"),
        number: value_t!(matches, "number", u32).unwrap(),
        opt_number: value_t!(matches, "opt-number", u32).ok(),
        width: value_t!(matches, "width", u32).unwrap(),
        free: Vec::new(),
    };

    if let Some(arg) = matches.value_of("input") {
        args.free.push(arg.to_string());
    }

    println!("{:#?}", args);
}
