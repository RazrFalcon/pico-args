/*!
An ultra simple CLI arguments parser.

- Only flags, options and free arguments are supported.
- Arguments can be separated by a space or `=`.
- Non UTF-8 arguments are supported.
- No help generation.
- No combined flags (like `-vvv` or `-abc`).
- Arguments are parsed in a linear order. From first to last.

## Example

```rust
use pico_args::Arguments;

struct Args {
    help: bool,
    version: bool,
    number: u32,
    opt_number: Option<u32>,
    width: u32,
    free: Vec<String>,
}

fn parse_width(s: &str) -> Result<u32, String> {
    s.parse().map_err(|_| "not a number".to_string())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = Arguments::from_env();
    // Arguments can be parsed in any order.
    let args = Args {
        // You can use a slice for multiple commands
        help: args.contains(["-h", "--help"]),
        // or just a string for a single one.
        version: args.contains("-V"),
        // Parses a value that implements `FromStr`.
        number: args.value_from_str("--number")?.unwrap_or(5),
        // Parses an optional value that implements `FromStr`.
        opt_number: args.value_from_str("--opt-number")?,
        // Parses a value using a specified function.
        width: args.value_from_fn("--width", parse_width)?.unwrap_or(10),
        // Will return all free arguments or an error if any flags are left.
        free: args.free()?,
    };

    Ok(())
}
```

## Alternatives

The core idea of `pico-args` is to provide some "sugar" for arguments parsing without
a lot of overhead (binary or compilation time wise).
There are no point in comparing parsing features, because `pico-args` supports
only the bare minimum. So we will compare only the size overhead and compilation time.

There are a lot of arguments parsing implementations, but we will use only these one:

- [clap](https://crates.io/crates/clap) - is the most popular and complete one
- [gumdrop](https://crates.io/crates/gumdrop) - a simple parser that uses procedural macros
- [structopt](https://crates.io/crates/structopt) - a two above combined

| | `pico-args` | `clap` | `gumdrop` | `structopt` |
---|---|---|---|---
| Binary overhead | 19.3KiB | 435.1KiB | 23.0KiB | 436.8KiB |
| Build time | 0.9s | 15s | 31s | 27s |
| Tested version | 0.1.0 | 2.33.0 | 0.6.0 | 0.2.18 |

- Binary size overhead was measured by subtracting the `.text` section size of an app with
  arguments parsing and a hello world app.
- Build time was measured using `hyperfine 'cargo clean; cargo build --release'`.
- Test projects can be found in `examples/`.
*/

#![doc(html_root_url = "https://docs.rs/pico-args/0.1.0")]

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::fmt::{self, Display};
use std::str::FromStr;
use std::ffi::{OsString, OsStr};


/// A list of possible errors.
#[derive(Clone, Debug)]
pub enum Error {
    /// Arguments must be a valid UTF-8 strings.
    NonUtf8Argument,

    /// An option without a value.
    OptionWithoutAValue(&'static str),

    /// Failed to parse a UTF-8 free-standing argument.
    #[allow(missing_docs)]
    Utf8ArgumentParsingFailed { value: String, cause: String },

    /// Failed to parse a raw free-standing argument.
    #[allow(missing_docs)]
    ArgumentParsingFailed { cause: String },

    /// Unused arguments left.
    UnusedArgsLeft(Vec<String>),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::NonUtf8Argument => {
                write!(f, "argument is not a UTF-8 string")
            }
            Error::OptionWithoutAValue(key) => {
                write!(f, "the '{}' option doesn't have an associated value", key)
            }
            Error::Utf8ArgumentParsingFailed { value, cause } => {
                write!(f, "failed to parse '{}' cause {}", value, cause)
            }
            Error::ArgumentParsingFailed { cause } => {
                write!(f, "failed to parse a binary argument cause {}", cause)
            }
            Error::UnusedArgsLeft(args) => {
                // Do not use `args.join()`, because it adds 1.2KiB.

                write!(f, "unused arguments left: ")?;
                for (i, arg) in args.iter().enumerate() {
                    write!(f, "{}", arg)?;

                    if i != args.len() - 1 {
                        write!(f, ", ")?;
                    }
                }

                Ok(())
            }
        }
    }
}

impl std::error::Error for Error {}


/// An arguments parser.
#[derive(Clone, Debug)]
pub struct Arguments(Vec<OsString>);

impl Arguments {
    /// Creates a parser from a vector of arguments.
    ///
    /// The executable path **must** be removed.
    pub fn from_vec(args: Vec<OsString>) -> Self {
        Arguments(args)
    }

    /// Creates a parser from `env::args()`.
    ///
    /// The executable path will be removed.
    pub fn from_env() -> Self {
        let mut args: Vec<_> = std::env::args_os().collect();
        args.remove(0);
        Arguments(args)
    }

    /// Checks that arguments contain a specified flag.
    ///
    /// Must be used only once for each flag.
    pub fn contains<A: Into<Keys>>(&mut self, keys: A) -> bool {
        self.contains_impl(keys.into())
    }

    #[inline(never)]
    fn contains_impl(&mut self, keys: Keys) -> bool {
        if let Some((idx, _)) = self.index_of(keys) {
            self.0.remove(idx);
            return true;
        }

        false
    }

    /// Parses a key-value pair using `FromStr` trait.
    ///
    /// This is a shorthand for `value_from_fn("--key", FromStr::from_str)`
    pub fn value_from_str<A, T>(&mut self, keys: A) -> Result<Option<T>, Error>
    where
        A: Into<Keys>,
        T: FromStr,
        <T as FromStr>::Err: Display,
    {
        self.value_from_fn(keys, FromStr::from_str)
    }

    /// Parses a key-value pair using a specified function.
    ///
    /// Must be used only once for each option.
    ///
    /// # Errors
    ///
    /// - When key or value is not a UTF-8 string. Use `value_from_os_str` instead.
    /// - When value parsing failed.
    /// - When key-value pair is separated not by space or `=`.
    pub fn value_from_fn<A: Into<Keys>, T, E: Display>(
        &mut self,
        keys: A,
        f: fn(&str) -> Result<T, E>,
    ) -> Result<Option<T>, Error> {
        self.value_from_fn_impl(keys.into(), f)
    }

    #[inline(never)]
    fn value_from_fn_impl<T, E: Display>(
        &mut self,
        keys: Keys,
        f: fn(&str) -> Result<T, E>,
    ) -> Result<Option<T>, Error> {
        if let Some((idx, key)) = self.index_of(keys) {
            // Parse a `--key value` pair.

            let value = self.get_str(key, idx + 1)?;
            match f(value) {
                Ok(value) => {
                    // Remove only when all checks are passed.
                    self.0.remove(idx);
                    self.0.remove(idx);
                    Ok(Some(value))
                }
                Err(e) => {
                    Err(Error::Utf8ArgumentParsingFailed {
                        value: value.to_string(),
                        cause: error_to_string(e),
                    })
                }
            }
        } else if let Some((idx, key)) = self.index_of2(keys) {
            // Parse a `--key=value` pair.

            let value = self.0.remove(idx);

            // Only UTF-8 strings are supported in this method.
            let value = value.to_str().ok_or_else(|| Error::NonUtf8Argument)?;

            let mut value_range = key.len()..value.len();
            if value.as_bytes().get(value_range.start) == Some(&b'=') {
                value_range.start += 1;
            } else {
                // Key must be followed by `=`.
                return Err(Error::OptionWithoutAValue(key));
            }

            // Check for quoted value.
            if let Some(c) = value.as_bytes().get(value_range.start).cloned() {
                if c == b'"' || c == b'\'' {
                    value_range.start += 1;

                    // A closing quote must be the same as an opening one.
                    if ends_with(&value[value_range.start..], c) {
                        value_range.end -= 1;
                    } else {
                        return Err(Error::OptionWithoutAValue(key));
                    }
                }
            }

            // Check length, otherwise String::drain will panic.
            if value_range.end - value_range.start == 0 {
                return Err(Error::OptionWithoutAValue(key));
            }

            // Extract `value` from `--key="value"`.
            let value = &value[value_range];

            if value.is_empty() {
                return Err(Error::OptionWithoutAValue(key));
            }

            match f(&value) {
                Ok(value) => Ok(Some(value)),
                Err(e) => Err(Error::Utf8ArgumentParsingFailed {
                    value: value.to_string(),
                    cause: error_to_string(e),
                }),
            }
        } else {
            Ok(None)
        }
    }

    /// Parses a key-value pair using a specified function.
    ///
    /// Unlike `value_from_fn`, parses `&OsStr` and not `&str`.
    ///
    /// Must be used only once for each option.
    ///
    /// # Errors
    ///
    /// - When value parsing failed.
    /// - When key-value pair is separated not by space.
    ///   Only `value_from_fn` supports `=` separator.
    pub fn value_from_os_str<A: Into<Keys>, T, E: Display>(
        &mut self,
        keys: A,
        f: fn(&OsStr) -> Result<T, E>,
    ) -> Result<Option<T>, Error> {
        self.value_from_os_str_impl(keys.into(), f)
    }

    #[inline(never)]
    fn value_from_os_str_impl<T, E: Display>(
        &mut self,
        keys: Keys,
        f: fn(&OsStr) -> Result<T, E>,
    ) -> Result<Option<T>, Error> {
        if let Some((idx, key)) = self.index_of(keys) {
            // Parse a `--key value` pair.

            let value = match self.0.get(idx + 1) {
                Some(v) => v,
                None => return Err(Error::OptionWithoutAValue(key)),
            };

            match f(value) {
                Ok(value) => {
                    // Remove only when all checks are passed.
                    self.0.remove(idx);
                    self.0.remove(idx);
                    Ok(Some(value))
                }
                Err(e) => {
                    Err(Error::ArgumentParsingFailed { cause: error_to_string(e) })
                }
            }
        } else {
            Ok(None)
        }
    }

    #[inline(never)]
    fn index_of<'a>(&self, keys: Keys) -> Option<(usize, &'a str)> {
        for key in &keys.0 {
            if !key.is_empty() {
                if let Some(i) = self.0.iter().position(|v| v == key) {
                    return Some((i, key));
                }
            }
        }

        None
    }

    #[inline(never)]
    fn index_of2<'a>(&self, keys: Keys) -> Option<(usize, &'a str)> {
        for key in &keys.0 {
            if !key.is_empty() {
                if let Some(i) = self.0.iter().position(|v| os_starts_with(v, key)) {
                    return Some((i, key));
                }
            }
        }

        None
    }

    #[inline(never)]
    fn get_str(&self, key: &'static str, idx: usize) -> Result<&str, Error> {
        let value = match self.0.get(idx) {
            Some(v) => v,
            None => return Err(Error::OptionWithoutAValue(key)),
        };

        os_to_str(value)
    }

    /// Parses a free-standing argument using `FromStr` trait.
    ///
    /// This is a shorthand for `free_from_fn(FromStr::from_str)`
    pub fn free_from_str<T>(&mut self) -> Result<Option<T>, Error>
        where
            T: FromStr,
            <T as FromStr>::Err: Display,
    {
        self.free_from_fn(FromStr::from_str)
    }

    /// Parses a free-standing argument using a specified function.
    ///
    /// Must be used only once for each argument.
    ///
    /// # Errors
    ///
    /// - When any flags are left.
    /// - When argument is not a UTF-8 string. Use `free_from_os_str` instead.
    /// - When value parsing failed.
    #[inline(never)]
    pub fn free_from_fn<T, E: Display>(
        &mut self,
        f: fn(&str) -> Result<T, E>,
    ) -> Result<Option<T>, Error> {
        self.check_for_flags()?;

        if self.0.is_empty() {
            Ok(None)
        } else {
            // A simple take_first() implementation.
            let mut value = OsString::new();
            std::mem::swap(self.0.first_mut().unwrap(), &mut value);
            self.0.remove(0);

            let value = os_to_str(value.as_os_str())?;
            match f(&value) {
                Ok(value) => Ok(Some(value)),
                Err(e) => Err(Error::Utf8ArgumentParsingFailed {
                    value: value.to_string(),
                    cause: error_to_string(e),
                }),
            }
        }
    }

    /// Parses a free-standing argument using a specified function.
    ///
    /// Must be used only once for each argument.
    ///
    /// # Errors
    ///
    /// - When any flags are left.
    /// - When value parsing failed.
    #[inline(never)]
    pub fn free_from_os_str<T, E: Display>(
        &mut self,
        f: fn(&OsStr) -> Result<T, E>,
    ) -> Result<Option<T>, Error> {
        self.check_for_flags()?;

        if self.0.is_empty() {
            Ok(None)
        } else {
            // A simple take_first() implementation.
            let mut value = OsString::new();
            std::mem::swap(self.0.first_mut().unwrap(), &mut value);
            self.0.remove(0);

            match f(value.as_os_str()) {
                Ok(value) => Ok(Some(value)),
                Err(e) => Err(Error::ArgumentParsingFailed { cause: error_to_string(e) }),
            }
        }
    }

    /// Returns a list of free arguments as Strings.
    ///
    /// This list will also include `-`, which indicates stdin.
    ///
    /// # Errors
    ///
    /// - When any flags are left.
    /// - When any of the arguments is not a UTF-8 string.
    pub fn free(self) -> Result<Vec<String>, Error> {
        self.check_for_flags()?;

        let mut args = Vec::new();
        for arg in self.0 {
            let arg = os_to_str(arg.as_os_str())?.to_string();
            args.push(arg);
        }

        Ok(args)
    }

    /// Returns a list of free arguments as OsStrings.
    ///
    /// This list will also include `-`, which indicates stdin.
    ///
    /// # Errors
    ///
    /// - When any flags are left.
    ///   Only UTF-8 strings will be checked for flag prefixes.
    pub fn free_os(self) -> Result<Vec<OsString>, Error> {
        self.check_for_flags()?;
        Ok(self.0)
    }

    #[inline(never)]
    fn check_for_flags(&self) -> Result<(), Error> {
        // Check that there are no flags left.
        // But allow `-` which is used to indicate stdin.
        let mut flags_left = Vec::new();
        for arg in &self.0 {
            if let Some(s) = arg.to_str() {
                if s.starts_with('-') && s != "-" {
                    flags_left.push(s.to_string());
                }
            }
        }

        if flags_left.is_empty() {
            Ok(())
        } else {
            Err(Error::UnusedArgsLeft(flags_left))
        }
    }

    /// Checks that all flags were processed.
    ///
    /// Use it instead of `free()` if you do not expect any free arguments.
    pub fn finish(self) -> Result<(), Error> {
        if !self.0.is_empty() {
            let mut args = Vec::new();
            for arg in &self.0 {
                if let Some(s) = arg.to_str() {
                    args.push(s.to_string());
                } else {
                    args.push("binary data".to_string());
                }
            }

            return Err(Error::UnusedArgsLeft(args));
        }

        Ok(())
    }
}

// Display::to_string() is usually inlined, so by wrapping it in a non-inlined
// function we are reducing the size a bit.
#[inline(never)]
fn error_to_string<E: Display>(e: E) -> String {
    e.to_string()
}

#[inline(never)]
fn os_starts_with(text: &OsStr, prefix: &str) -> bool {
    match text.to_str() {
        Some(s) => s.get(0..prefix.len()) == Some(prefix),
        None => false,
    }
}

#[inline(never)]
fn ends_with(text: &str, c: u8) -> bool {
    if text.is_empty() {
        false
    } else {
        text.as_bytes()[text.len() - 1] == c
    }
}

#[inline(never)]
fn os_to_str(text: &OsStr) -> Result<&str, Error> {
    text.to_str().ok_or_else(|| Error::NonUtf8Argument)
}


/// A keys container.
///
/// Should not be used directly.
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct Keys([&'static str; 2]);

impl From<[&'static str; 2]> for Keys {
    #[inline]
    fn from(v: [&'static str; 2]) -> Self {
        debug_assert!(v[0].starts_with("-"), "an argument should start with '-'");
        debug_assert!(!v[0].starts_with("--"), "the first argument should be short");
        debug_assert!(v[1].starts_with("--"), "the second argument should be long");

        Keys(v)
    }
}

impl From<&'static str> for Keys {
    #[inline]
    fn from(v: &'static str) -> Self {
        debug_assert!(v.starts_with("-"), "an argument should start with '-'");

        Keys([v, ""])
    }
}
