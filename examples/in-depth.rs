fn help_print() {
    println!(
    "archive {}

USAGE:
    archive <SUBCOMMAND>

FLAGS:
    -h,--help       Prints help information
    -V,--version    Prints version information

SUBCOMMANDS:
    create     Create archive
    extract    Extract archive
    help       Prints this message or the help of the given subcommand(s)
    keygen     Generate keys
    list       List archive",
        env!("CARGO_PKG_VERSION")
    );
}

fn version_print() {
    println!("archive {}", env!("CARGO_PKG_VERSION"))
}

#[derive(Debug)]
enum AppArgs {
    Flags {
        help: bool,
        version: bool
    },
    Create {
        secret: String,
        file: String,
        folder: Vec<String>
    },
    Extract {
        public: String,
        file: String,
        folder: Vec<String>
    },
    Keygen {
        public: String,
        secret: String,
    },
    List {
        public: String,
        file: String,
    }
}

fn main() {
    match submain() {
        AppArgs::Flags { help, version } => {
            if help {
                help_print();
            } else if version {
                version_print();
            } else {
                help_print();
            }
        },
        AppArgs::Create { file, secret, folder } => {
            println!("File: {} Secret: {} Folder: {:#?}",&file, &secret, &folder);
        },
        AppArgs::Extract{public, file, folder} => {
            println!("Public: {} File: {} Folder: {:#?}",&public,  &file, &folder);
        },
        AppArgs::Keygen{public,secret} => {
            println!("Secret: {} Public: {}",&secret, &public)
        },
        AppArgs::List{public,file} => {
            println!("Public: {} File: {}", &public,  &file) 
        },
    }
}

fn submain() -> AppArgs {
    let mut args = pico_args::Arguments::from_env();

    let subcommand = match args.subcommand() {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Error: Invalid subcommand.");
            std::process::exit(1);
        }
    };

    match subcommand.as_deref() {
        Some("create") => match parse_create(args) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("\
                    Error: {}\n\n\
                    USAGE:\n\
                    \tarchive create <folder> --file <file> --secret <secret>\n\n\
                    For more information try --help", e);
                std::process::exit(1);
            }
        },
        Some("extract") => match parse_extract(args) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("\
                    Error: {}\n\n\
                    USAGE:\n\
                    \tarchive extract <folder> --file <file> --public <public>\n\n\
                    For more information try --help", e);
                std::process::exit(1);               
            }
        },
        Some("keygen") => match parse_keygen(args) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("\
                    Error: {}\n\n\
                    USAGE:\n\
                    \tarchive keygen <folder> --secret <secret> --secret <secret>\n\n\
                    For more information try --help", e);
                    std::process::exit(1);
            }
        },
        Some("list") => match parse_list(args) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("\
                    Error: {}\n\n\
                    USAGE:\n\
                    \tarchive list <folder> --public <public> --file <file>\n\n\
                    For more information try --help", e);
                std::process::exit(1);
            }
        },
        None => {
            match parse_flags(args) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("\
                        Error: {}\n\n\
                        USAGE:\n\
                        \tarchive --help", e);
                    std::process::exit(1);
                }
            }
        }
        Some(s) => {
            eprintln!("Error: '{}' is an unknown subcommand.", s);
            std::process::exit(1);
        }
    }
}

// Determines how the create subcommand's arguements are parsed
fn parse_create(mut args: pico_args::Arguments) -> Result<AppArgs, pico_args::Error> {
    let app_args = AppArgs::Create {
        file: args.value_from_str("--file")?,
        secret: args.value_from_str("--secret")?,
        folder: args.free()?,
    };
    Ok(app_args)
}

// Determines how the extract subcommand's arguements are parsed
fn parse_extract(mut args: pico_args::Arguments) -> Result<AppArgs, pico_args::Error> {
    let app_args = AppArgs::Extract {
        file: args.value_from_str("--file")?,
        public: args.value_from_str("--public")?,
        folder: args.free()?,
    };
    Ok(app_args)
}

// Determines how the keygen subcommand's arguements are parsed
fn parse_keygen(mut args: pico_args::Arguments) -> Result<AppArgs, pico_args::Error> {
    let app_args = AppArgs::Keygen {
        public: args.value_from_str("--public")?,
        secret: args.value_from_str("--secret")?,
    };

    args.finish()?;
    Ok(app_args)
}

// Determines how the list subcommand's arguements are parsed
fn parse_list(mut args: pico_args::Arguments) -> Result<AppArgs, pico_args::Error> {
    let app_args = AppArgs::List {
        file: args.value_from_str("--file")?,
            public: args.value_from_str("--secret")?,
    };

    args.finish()?;
    Ok(app_args)
}

// Determines how the flags are parsed
fn parse_flags(mut args: pico_args::Arguments) -> Result<AppArgs, pico_args::Error> {
    let app_args = AppArgs::Flags {
        help: args.contains(["-h", "--help"]),
        version: args.contains(["-V", "--version"])
    };

    args.finish()?;
    Ok(app_args)
}
