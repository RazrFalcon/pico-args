use std::str::FromStr;
use std::ffi::OsString;

use pico_args::*;

fn to_vec(args: &[&str]) -> Vec<OsString> {
    args.iter().map(|s| s.to_string().into()).collect()
}

#[test]
fn no_args() {
    let _ = Arguments::from_vec(to_vec(&[]));
}

#[test]
fn single_short_contains() {
    let mut args = Arguments::from_vec(to_vec(&["-V"]));
    assert!(args.contains("-V"));
}

#[test]
fn single_long_contains() {
    let mut args = Arguments::from_vec(to_vec(&["--version"]));
    assert!(args.contains("--version"));
}

#[test]
fn contains_two_01() {
    let mut args = Arguments::from_vec(to_vec(&["--version"]));
    assert!(args.contains(["-v", "--version"]));
}

#[test]
fn contains_two_02() {
    let mut args = Arguments::from_vec(to_vec(&["-v"]));
    assert!(args.contains(["-v", "--version"]));
}

#[test]
fn contains_two_03() {
    let mut args = Arguments::from_vec(to_vec(&["-v", "--version"]));
    assert!(args.contains(["-v", "--version"]));
}

#[test]
#[should_panic]
fn invalid_flag_01() {
    let mut args = Arguments::from_vec(to_vec(&["-v", "--version"]));
    assert!(args.contains("v"));
}

#[test]
#[should_panic]
fn invalid_flag_02() {
    let mut args = Arguments::from_vec(to_vec(&["-v", "--version"]));
    assert!(args.contains(["v", "--version"]));
}

#[test]
#[should_panic]
fn invalid_flag_03() {
    let mut args = Arguments::from_vec(to_vec(&["-v", "--version"]));
    assert!(args.contains(["-v", "-version"]));
}

#[test]
#[should_panic]
fn invalid_flag_04() {
    let mut args = Arguments::from_vec(to_vec(&["-v", "--version"]));
    assert!(args.contains(["-v", "version"]));
}


#[test]
fn option_01() {
    let mut args = Arguments::from_vec(to_vec(&["-w", "10"]));
    let value: Option<u32> = args.opt_value_from_str("-w").unwrap();
    assert_eq!(value.unwrap(), 10);
}

#[test]
fn option_02() {
    let mut args = Arguments::from_vec(to_vec(&["--width", "10"]));
    let value: Option<u32> = args.opt_value_from_str("--width").unwrap();
    assert_eq!(value.unwrap(), 10);
}

#[test]
fn option_03() {
    let mut args = Arguments::from_vec(to_vec(&["--name", "test"]));
    let value: Option<String> = args.opt_value_from_str("--name").unwrap();
    assert_eq!(value.unwrap(), "test");
}

#[test]
fn eq_option_01() {
    let mut args = Arguments::from_vec(to_vec(&["-w=10"]));
    let value: Option<u32> = args.opt_value_from_str("-w").unwrap();
    assert_eq!(value.unwrap(), 10);
}

#[test]
fn eq_option_02() {
    let mut args = Arguments::from_vec(to_vec(&["-w='10'"]));
    let value: Option<u32> = args.opt_value_from_str("-w").unwrap();
    assert_eq!(value.unwrap(), 10);
}

#[test]
fn eq_option_03() {
    let mut args = Arguments::from_vec(to_vec(&["-w=\"10\""]));
    let value: Option<u32> = args.opt_value_from_str("-w").unwrap();
    assert_eq!(value.unwrap(), 10);
}

#[test]
fn eq_option_04() {
    let mut args = Arguments::from_vec(to_vec(&["--width2=15", "--width=10"]));
    let value: Option<u32> = args.opt_value_from_str("--width").unwrap();
    assert_eq!(value.unwrap(), 10);
}

#[test]
fn eq_option_err_01() {
    let mut args = Arguments::from_vec(to_vec(&["-w="]));
    let value: Result<Option<u32>, Error> = args.opt_value_from_str("-w");
    assert_eq!(value.unwrap_err().to_string(),
               "the '-w' option doesn't have an associated value");
}

#[test]
fn eq_option_err_02() {
    let mut args = Arguments::from_vec(to_vec(&["-w='"]));
    let value: Result<Option<u32>, Error> = args.opt_value_from_str("-w");
    assert_eq!(value.unwrap_err().to_string(),
               "the '-w' option doesn't have an associated value");
}

#[test]
fn eq_option_err_03() {
    let mut args = Arguments::from_vec(to_vec(&["-w=''"]));
    let value: Result<Option<u32>, Error> = args.opt_value_from_str("-w");
    assert_eq!(value.unwrap_err().to_string(),
               "the '-w' option doesn't have an associated value");
}

#[test]
fn eq_option_err_04() {
    let mut args = Arguments::from_vec(to_vec(&["-w='\""]));
    let value: Result<Option<u32>, Error> = args.opt_value_from_str("-w");
    assert_eq!(value.unwrap_err().to_string(),
               "the '-w' option doesn't have an associated value");
}

#[test]
fn eq_option_err_05() {
    let mut args = Arguments::from_vec(to_vec(&["-w='10\""]));
    let value: Result<Option<u32>, Error> = args.opt_value_from_str("-w");
    assert_eq!(value.unwrap_err().to_string(),
               "the '-w' option doesn't have an associated value");
}

#[test]
fn eq_option_err_06() {
    let mut args = Arguments::from_vec(to_vec(&["-w-10"]));
    let value: Result<Option<u32>, Error> = args.opt_value_from_str("-w");
    assert_eq!(value.unwrap(), None);
}

#[test]
fn eq_option_err_07() {
    let mut args = Arguments::from_vec(to_vec(&["-w=a"]));
    let value: Result<Option<u32>, Error> = args.opt_value_from_str("-w");
    assert_eq!(value.unwrap_err().to_string(),
               "failed to parse 'a' cause invalid digit found in string");
}

#[test]
fn duplicated_options_01() {
    let mut args = Arguments::from_vec(to_vec(&["--name", "test1", "--name", "test2"]));
    let value1: Option<String> = args.opt_value_from_str("--name").unwrap();
    let value2: Option<String> = args.opt_value_from_str("--name").unwrap();
    assert_eq!(value1.unwrap(), "test1");
    assert_eq!(value2.unwrap(), "test2");
}

#[test]
fn option_from_os_str_01() {
    use std::path::PathBuf;

    fn parse_path(s: &std::ffi::OsStr) -> Result<PathBuf, &'static str> {
        Ok(s.into())
    }

    let mut args = Arguments::from_vec(to_vec(&["--input", "text.txt"]));
    let value: Result<Option<PathBuf>, Error> = args.opt_value_from_os_str("--input", parse_path);
    assert_eq!(value.unwrap().unwrap().display().to_string(), "text.txt");
}

#[test]
fn missing_option_value_01() {
    let mut args = Arguments::from_vec(to_vec(&["--value"]));
    let value: Result<Option<u32>, Error> = args.opt_value_from_str("--value");
    assert_eq!(value.unwrap_err().to_string(),
               "the '--value' option doesn't have an associated value");
}

#[test]
fn missing_option_value_02() {
    let mut args = Arguments::from_vec(to_vec(&["--value"]));
    let value: Result<Option<u32>, Error> = args.opt_value_from_str("--value");
    assert!(value.is_err()); // ignore error
    // the `--value` flag should not be removed by the previous command
    assert!(args.finish().is_err());
}

#[test]
fn missing_option_value_03() {
    let mut args = Arguments::from_vec(to_vec(&["--value", "q"]));
    let value: Result<Option<u32>, Error> = args.opt_value_from_str("--value");
    assert!(value.is_err()); // ignore error
    // the `--value` flag should not be removed by the previous command
    assert!(args.finish().is_err());
}

#[test]
fn free_01() {
    let args = Arguments::from_vec(to_vec(&[]));
    assert_eq!(args.free_os().unwrap(), to_vec(&[]));
}

#[test]
fn free_02() {
    let args = Arguments::from_vec(to_vec(&["text.txt"]));
    assert_eq!(args.free_os().unwrap(), to_vec(&["text.txt"]));
}

#[test]
fn free_03() {
    let args = Arguments::from_vec(to_vec(&["text.txt", "text2.txt"]));
    assert_eq!(args.free_os().unwrap(), to_vec(&["text.txt", "text2.txt"]));
}

#[test]
fn free_04() {
    let mut args = Arguments::from_vec(to_vec(&["-h", "text.txt", "text2.txt"]));
    assert!(args.contains("-h"));
    assert_eq!(args.free_os().unwrap(), to_vec(&["text.txt", "text2.txt"]));
}

#[test]
fn free_05() {
    let mut args = Arguments::from_vec(to_vec(&["text.txt", "-h", "text2.txt"]));
    assert!(args.contains("-h"));
    assert_eq!(args.free_os().unwrap(), to_vec(&["text.txt", "text2.txt"]));
}

#[test]
fn free_06() {
    let mut args = Arguments::from_vec(to_vec(&["text.txt", "text2.txt", "-h"]));
    assert!(args.contains("-h"));
    assert_eq!(args.free_os().unwrap(), to_vec(&["text.txt", "text2.txt"]));
}

#[test]
fn free_from_fn_01() {
    let mut args = Arguments::from_vec(to_vec(&["5"]));
    assert_eq!(args.free_from_fn(u32::from_str).unwrap(), Some(5));
}

#[test]
fn free_from_fn_02() {
    let mut args = Arguments::from_vec(to_vec(&[]));
    assert_eq!(args.free_from_fn(u32::from_str).unwrap(), None);
}

#[test]
fn free_from_fn_03() {
    let mut args = Arguments::from_vec(to_vec(&["-h"]));
    assert_eq!(args.free_from_fn(u32::from_str).unwrap_err().to_string(),
               "unused arguments left: -h");
}

#[test]
fn free_from_fn_04() {
    let mut args = Arguments::from_vec(to_vec(&["a"]));
    assert_eq!(args.free_from_fn(u32::from_str).unwrap_err().to_string(),
               "failed to parse 'a' cause invalid digit found in string");
}

#[test]
fn free_from_str_01() {
    let mut args = Arguments::from_vec(to_vec(&["5"]));
    let value: Result<Option<u32>, Error> = args.free_from_str();
    assert_eq!(value.unwrap(), Some(5));
}

#[test]
fn unused_args_01() {
    let args = Arguments::from_vec(to_vec(&["-h", "text.txt"]));
    assert_eq!(args.finish().unwrap_err().to_string(),
               "unused arguments left: -h, text.txt");
}

#[test]
fn unused_args_02() {
    let args = Arguments::from_vec(to_vec(&["-h", "text.txt"]));
    assert_eq!(args.free().unwrap_err().to_string(),
               "unused arguments left: -h");
}

#[test]
fn stdin() {
    let args = Arguments::from_vec(to_vec(&["-"]));
    assert_eq!(args.free_os().unwrap(), to_vec(&["-"]));
}

#[test]
fn required_option_01() {
    let mut args = Arguments::from_vec(to_vec(&["--width", "10"]));
    let value: u32 = args.value_from_str("--width").unwrap();
    assert_eq!(value, 10);
}

#[test]
fn missing_required_option_01() {
    let mut args = Arguments::from_vec(to_vec(&[]));
    let value: Result<u32, Error> = args.value_from_str("-w");
    assert_eq!(value.unwrap_err().to_string(),
               "the '-w' option must be set");
}

#[test]
fn missing_required_option_02() {
    let mut args = Arguments::from_vec(to_vec(&[]));
    let value: Result<u32, Error> = args.value_from_str("--width");
    assert_eq!(value.unwrap_err().to_string(),
               "the '--width' option must be set");
}

#[test]
fn missing_required_option_03() {
    let mut args = Arguments::from_vec(to_vec(&[]));
    let value: Result<u32, Error> = args.value_from_str(["-w", "--width"]);
    assert_eq!(value.unwrap_err().to_string(),
               "the '-w/--width' option must be set");
}

#[test]
fn subcommand() {
    let mut args = Arguments::from_vec(to_vec(&["toolchain", "install", "--help"]));

    let cmd = args.subcommand().unwrap();
    assert_eq!(cmd, Some("toolchain".to_string()));

    let cmd = args.subcommand().unwrap();
    assert_eq!(cmd, Some("install".to_string()));

    let cmd = args.subcommand().unwrap();
    assert_eq!(cmd, None);
}
