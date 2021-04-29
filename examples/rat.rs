// Rat - A simple and practical example of pico_args
use pico_args::{Arguments, Error};
use std::fs;

// Help message
const HELP: &str = "\
rat - A program to read and display files

Usage:
    rat [options] FILE

Options:
    -h, --help     | Display this help message
    -l, --lines    | Display line numbers

Examples:
    rat note12.txt | Displays the file note12.txt without line numbers
    rat -l test.py | Displays the file test.py with line numbers
";

fn main() {
    // Read arguments and handle any errors
    if let Err(error) = do_args() {
        println!("Error: {}", error);
    }
}

fn do_args() -> Result<(), Error> {
    // Create argument reader from user provided arguments
    let mut args = Arguments::from_env();
    // NOTE: It's best practice to handle flags before everything else
    // Check for a help flag (and exit if it's there)
    if args.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }
    // Check for the presence of the --lines flag
    let line_numbers = args.contains(["-l", "--lines"]);
    // Read file and print it
    let path = args.free_from_str::<String>()?;
    if let Ok(contents) = fs::read_to_string(&path) {
        // File was read sucessfully, print it
        do_print(contents, line_numbers);
    } else {
        // File failed to open
        println!("Error: failed to open file `{}`", path);
    }
    // Gather remaining arguments and warn the user about them
    let remaining = args.finish();
    if !remaining.is_empty() {
        eprintln!("Warning: unused arguments: {:?}.", remaining)
    }
    // Exit successfully
    Ok(())
}

fn do_print(contents: String, line_numbers: bool) {
    // Print out contents
    let mut lines = contents.split('\n');
    // Remove last \n
    lines.next_back();
    // Check for lines flag
    if line_numbers {
        // User provided the --lines flag
        for (number, line) in lines.enumerate() {
            // Print out lines
            println!("{: <5}| {}", number, line);
        }
    } else {
        // User didn't provide the --lines flag
        println!("{}", lines.collect::<Vec<_>>().join("\n"));
    }
}
