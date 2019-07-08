/*!
An ultra simple CLI arguments parser.

- Only flags, options and free arguments are supported.
- Arguments must be separated by a space. `=` isn't supported.
- No help generation.
- No combined flags (like `-vvv` or `-abc`).

## Alternatives

The core idea of `pico-args` is to provide some "sugar" for arguments parsing without
a lot of overhead (binary or compilation time wise).
There are no point in comparing parsing features, because `pico-args` supports
only the bare minimum. So we will compare only the size overhead and compilation time.

There are a lot of arguments parsing implementations, but we will use only two:

- [clap](https://crates.io/crates/clap) - is the most popular and complete one
- [gumdrop](https://crates.io/crates/gumdrop) - a simple parser that uses procedural macros

| Feature | `pico-args` | `clap` | `gumdrop` |
---|---|---|---
| Binary size | 5.7KiB | 340KiB | 12.9KiB |
| Build time | 0.8s | 15s | 31s |
| Tested version | 0.1.0 | 2.33.0 | 0.6.0 |

- Binary size overhead was measured using `cargo bloat --release --crates` (app + args crate).
- Build time was measured using `hyperfine 'cargo clean; cargo build --release'`.
- Test projects can be found in `examples/`.
*/

#![doc(html_root_url = "https://docs.rs/pico-args/0.1.0")]

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::fmt::{self, Display};
use std::str::FromStr;


/// A list of possible errors.
#[derive(Clone, Debug)]
pub enum Error {
    /// An option without a value.
    OptionWithoutAValue(&'static str),

    /// An option without a value.
    OptionValueParsingFailed(&'static str, String),

    /// Unused arguments left.
    UnusedArgsLeft(Vec<String>),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::OptionWithoutAValue(key) => {
                write!(f, "the '{}' option doesn't have an associated value", key)
            }
            Error::OptionValueParsingFailed(key, e) => {
                write!(f, "failed to parse a '{}' value cause {}", key, e)
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
pub struct Arguments(Vec<String>);

impl Arguments {
    /// Creates a parser from the vector of arguments.
    ///
    /// The executable path *must* be removed.
    pub fn from_args(args: Vec<String>) -> Self {
        Arguments(args)
    }

    /// Creates a parser from `env::args()`.
    ///
    /// The executable path will be removed.
    pub fn from_env() -> Self {
        let mut args: Vec<String> = std::env::args().collect();
        args.remove(0);
        Arguments(args)
    }

    /// Checks that arguments contains a specified flag.
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

    /// Parses a key-value pair.
    ///
    /// Key and value must be separated by a space and not `=`.
    ///
    /// Uses the `FromStr` trait for value conversion.
    ///
    /// Must be used only once for each option.
    pub fn value_from_str<A, T>(&mut self, keys: A) -> Result<Option<T>, Error>
    where
        A: Into<Keys>,
        T: FromStr,
        <T as FromStr>::Err: Display,
    {
        self.value_from_fn_impl(keys.into(), from_str_wrapper)
    }

    /// Parses a key-value pair.
    ///
    /// Key and value must be separated by a space and not `=`.
    ///
    /// Uses a specified function for value conversion.
    ///
    /// Must be used only once for each option.
    pub fn value_from_fn<A: Into<Keys>, T>(
        &mut self,
        keys: A,
        f: fn(&str) -> Result<T, String>,
    ) -> Result<Option<T>, Error> {
        self.value_from_fn_impl(keys.into(), f)
    }

    #[inline(never)]
    fn value_from_fn_impl<T>(
        &mut self,
        keys: Keys,
        f: fn(&str) -> Result<T, String>,
    ) -> Result<Option<T>, Error> {
        if let Some((idx, key)) = self.index_of(keys) {
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
                    Err(Error::OptionValueParsingFailed(key, e))
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

    /// Checks that all flags were processed.
    ///
    /// Use it instead of `free()` if you do not expect any free arguments.
    pub fn finish(self) -> Result<(), Error> {
        if !self.0.is_empty() {
            return Err(Error::UnusedArgsLeft(self.0));
        }

        Ok(())
    }

    /// Returns a list of free arguments.
    ///
    /// Will return an error, if any flags were left.
    pub fn free(self) -> Result<Vec<String>, Error> {
        // Check that no flags left.
        // But allow `-` which is used to indicate stdin.
        let flags_left: Vec<String> = self.0.iter()
            .filter(|s| *s != "-" && s.starts_with('-'))
            .map(String::from)
            .collect();

        if !flags_left.is_empty() {
            return Err(Error::UnusedArgsLeft(flags_left));
        }

        Ok(self.0)
    }
}

#[inline]
fn from_str_wrapper<T>(s: &str) -> Result<T, String>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    s.parse().map_err(|e: <T as FromStr>::Err| e.to_string())
}


/// A keys container.
///
/// Should not be used directly.
#[derive(Clone, Copy, Debug)]
pub struct Keys([&'static str; 2]);

impl From<[&'static str; 2]> for Keys {
    fn from(v: [&'static str; 2]) -> Self {
        debug_assert!(v[0].starts_with("-"), "an argument should start with '-'");
        debug_assert!(!v[0].starts_with("--"), "the first argument should be short");
        debug_assert!(v[1].starts_with("--"), "the second argument should be long");

        Keys(v)
    }
}

impl From<&'static str> for Keys {
    fn from(v: &'static str) -> Self {
        debug_assert!(v.starts_with("-"), "an argument should start with '-'");

        Keys([v, ""])
    }
}
