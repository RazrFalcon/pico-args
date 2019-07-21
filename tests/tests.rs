use pico_args::Arguments;

fn to_vec(args: &[&str]) -> Vec<String> {
    args.iter().map(|s| s.to_string()).collect()
}

#[test]
fn no_args() {
    let _ = Arguments::from_args(to_vec(&[]));
}

#[test]
fn single_short_contains() {
    let mut args = Arguments::from_args(to_vec(&["-V"]));
    assert!(args.contains("-V"));
}

#[test]
fn single_long_contains() {
    let mut args = Arguments::from_args(to_vec(&["--version"]));
    assert!(args.contains("--version"));
}

#[test]
fn contains_two_1() {
    let mut args = Arguments::from_args(to_vec(&["--version"]));
    assert!(args.contains(["-v", "--version"]));
}

#[test]
fn contains_two_2() {
    let mut args = Arguments::from_args(to_vec(&["-v"]));
    assert!(args.contains(["-v", "--version"]));
}

#[test]
fn contains_two_3() {
    let mut args = Arguments::from_args(to_vec(&["-v", "--version"]));
    assert!(args.contains(["-v", "--version"]));
}

#[test]
#[should_panic]
fn invalid_flag_1() {
    let mut args = Arguments::from_args(to_vec(&["-v", "--version"]));
    assert!(args.contains("v"));
}

#[test]
#[should_panic]
fn invalid_flag_2() {
    let mut args = Arguments::from_args(to_vec(&["-v", "--version"]));
    assert!(args.contains(["v", "--version"]));
}

#[test]
#[should_panic]
fn invalid_flag_3() {
    let mut args = Arguments::from_args(to_vec(&["-v", "--version"]));
    assert!(args.contains(["-v", "-version"]));
}

#[test]
#[should_panic]
fn invalid_flag_4() {
    let mut args = Arguments::from_args(to_vec(&["-v", "--version"]));
    assert!(args.contains(["-v", "version"]));
}


#[test]
fn option_01() {
    let mut args = Arguments::from_args(to_vec(&["-w", "10"]));
    let value: Option<u32> = args.value_from_str("-w").unwrap();
    assert_eq!(value.unwrap(), 10);
}

#[test]
fn option_02() {
    let mut args = Arguments::from_args(to_vec(&["--width", "10"]));
    let value: Option<u32> = args.value_from_str("--width").unwrap();
    assert_eq!(value.unwrap(), 10);
}

#[test]
fn option_03() {
    let mut args = Arguments::from_args(to_vec(&["--name", "test"]));
    let value: Option<String> = args.value_from_str("--name").unwrap();
    assert_eq!(value.unwrap(), "test");
}

#[test]
fn eq_option_01() {
    let mut args = Arguments::from_args(to_vec(&["-w=10"]));
    let value: Option<u32> = args.value_from_str("-w").unwrap();
    assert_eq!(value.unwrap(), 10);
}

#[test]
fn eq_option_02() {
    let mut args = Arguments::from_args(to_vec(&["-w='10'"]));
    let value: Option<u32> = args.value_from_str("-w").unwrap();
    assert_eq!(value.unwrap(), 10);
}

#[test]
fn eq_option_03() {
    let mut args = Arguments::from_args(to_vec(&["-w=\"10\""]));
    let value: Option<u32> = args.value_from_str("-w").unwrap();
    assert_eq!(value.unwrap(), 10);
}

#[test]
fn eq_option_err_01() {
    let mut args = Arguments::from_args(to_vec(&["-w="]));
    let value: Result<Option<u32>, pico_args::Error> = args.value_from_str("-w");
    assert_eq!(value.unwrap_err().to_string(),
               "the '-w' option doesn't have an associated value");
}

#[test]
fn eq_option_err_02() {
    let mut args = Arguments::from_args(to_vec(&["-w='"]));
    let value: Result<Option<u32>, pico_args::Error> = args.value_from_str("-w");
    assert_eq!(value.unwrap_err().to_string(),
               "the '-w' option doesn't have an associated value");
}

#[test]
fn eq_option_err_03() {
    let mut args = Arguments::from_args(to_vec(&["-w=''"]));
    let value: Result<Option<u32>, pico_args::Error> = args.value_from_str("-w");
    assert_eq!(value.unwrap_err().to_string(),
               "the '-w' option doesn't have an associated value");
}

#[test]
fn eq_option_err_04() {
    let mut args = Arguments::from_args(to_vec(&["-w='\""]));
    let value: Result<Option<u32>, pico_args::Error> = args.value_from_str("-w");
    assert_eq!(value.unwrap_err().to_string(),
               "the '-w' option doesn't have an associated value");
}

#[test]
fn eq_option_err_05() {
    let mut args = Arguments::from_args(to_vec(&["-w='10\""]));
    let value: Result<Option<u32>, pico_args::Error> = args.value_from_str("-w");
    assert_eq!(value.unwrap_err().to_string(),
               "the '-w' option doesn't have an associated value");
}

#[test]
fn eq_option_err_06() {
    let mut args = Arguments::from_args(to_vec(&["-w-10"]));
    let value: Result<Option<u32>, pico_args::Error> = args.value_from_str("-w");
    assert_eq!(value.unwrap_err().to_string(),
               "the '-w' option doesn't have an associated value");
}

#[test]
fn missing_option_value_1() {
    let mut args = Arguments::from_args(to_vec(&["--value"]));
    let value: Result<Option<u32>, pico_args::Error> = args.value_from_str("--value");
    assert_eq!(value.unwrap_err().to_string(),
               "the '--value' option doesn't have an associated value");
}

#[test]
fn missing_option_value_2() {
    let mut args = Arguments::from_args(to_vec(&["--value"]));
    let value: Result<Option<u32>, pico_args::Error> = args.value_from_str("--value");
    assert!(value.is_err()); // ignore error
    // the `--value` flag should not be removed by the previous command
    assert!(args.finish().is_err());
}

#[test]
fn missing_option_value_3() {
    let mut args = Arguments::from_args(to_vec(&["--value", "q"]));
    let value: Result<Option<u32>, pico_args::Error> = args.value_from_str("--value");
    assert!(value.is_err()); // ignore error
    // the `--value` flag should not be removed by the previous command
    assert!(args.finish().is_err());
}

#[test]
fn free_1() {
    let args = Arguments::from_args(to_vec(&[]));
    assert_eq!(args.free().unwrap(), to_vec(&[]));
}

#[test]
fn free_2() {
    let args = Arguments::from_args(to_vec(&["text.txt"]));
    assert_eq!(args.free().unwrap(), to_vec(&["text.txt"]));
}

#[test]
fn free_3() {
    let args = Arguments::from_args(to_vec(&["text.txt", "text2.txt"]));
    assert_eq!(args.free().unwrap(), to_vec(&["text.txt", "text2.txt"]));
}

#[test]
fn free_4() {
    let mut args = Arguments::from_args(to_vec(&["-h", "text.txt", "text2.txt"]));
    assert!(args.contains("-h"));
    assert_eq!(args.free().unwrap(), to_vec(&["text.txt", "text2.txt"]));
}

#[test]
fn free_5() {
    let mut args = Arguments::from_args(to_vec(&["text.txt", "-h", "text2.txt"]));
    assert!(args.contains("-h"));
    assert_eq!(args.free().unwrap(), to_vec(&["text.txt", "text2.txt"]));
}

#[test]
fn free_6() {
    let mut args = Arguments::from_args(to_vec(&["text.txt", "text2.txt", "-h"]));
    assert!(args.contains("-h"));
    assert_eq!(args.free().unwrap(), to_vec(&["text.txt", "text2.txt"]));
}

#[test]
fn unused_args_1() {
    let args = Arguments::from_args(to_vec(&["-h", "text.txt"]));
    assert_eq!(args.finish().unwrap_err().to_string(),
               "unused arguments left: -h, text.txt");
}

#[test]
fn unused_args_2() {
    let args = Arguments::from_args(to_vec(&["-h", "text.txt"]));
    assert_eq!(args.free().unwrap_err().to_string(),
               "unused arguments left: -h");
}

#[test]
fn stdin() {
    let args = Arguments::from_args(to_vec(&["-"]));
    assert_eq!(args.free().unwrap(), to_vec(&["-"]));
}
