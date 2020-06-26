/*!
An ultra simple CLI arguments parser.

- Only flags, options, free arguments and subcommands are supported.
- Arguments can be separated by a space or `=`.
- Non UTF-8 arguments are supported.
- No help generation.
- No combined flags (like `-vvv`, `-abc` or `-j1`).
- Arguments are parsed in a linear order. From first to last.

## Example

```rust
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
    let mut args = pico_args::Arguments::from_env();
    // Arguments can be parsed in any order.
    let args = Args {
        // You can use a slice for multiple commands
        help: args.contains(["-h", "--help"]),
        // or just a string for a single one.
        version: args.contains("-V"),
        // Parses an optional value that implements `FromStr`.
        number: args.opt_value_from_str("--number")?.unwrap_or(5),
        // Parses an optional value that implements `FromStr`.
        opt_number: args.opt_value_from_str("--opt-number")?,
        // Parses an optional value using a specified function.
        width: args.opt_value_from_fn("--width", parse_width)?.unwrap_or(10),
        // Will return all free arguments or an error if any flags are left.
        free: args.free()?,
    };

    Ok(())
}
```

## Build features

- `eq-separator`

  Allows parsing arguments separated by `=`. Enabled by default.<br/>
  This feature adds about 1KiB to the resulting binary.
*/

#![doc(html_root_url = "https://docs.rs/pico-args/0.3.3")]

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::ffi::{OsString, OsStr};
use std::fmt::{self, Display};
use std::str::FromStr;


/// A list of possible errors.
#[derive(Clone, Debug)]
pub enum Error {
    /// Arguments must be a valid UTF-8 strings.
    NonUtf8Argument,

    /// A missing option.
    MissingOption(Keys),

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
            Error::MissingOption(key) => {
                if key.second().is_empty() {
                    write!(f, "the '{}' option must be set", key.first())
                } else {
                    write!(f, "the '{}/{}' option must be set", key.first(), key.second())
                }
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


#[derive(Clone, Copy, PartialEq)]
enum PairKind {
    #[cfg(feature = "eq-separator")]
    SingleArgument,
    TwoArguments,
}


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

    /// Creates a parser from [`env::args`].
    ///
    /// The executable path will be removed.
    ///
    /// [`env::args`]: https://doc.rust-lang.org/stable/std/env/fn.args.html
    pub fn from_env() -> Self {
        let mut args: Vec<_> = std::env::args_os().collect();
        args.remove(0);
        Arguments(args)
    }

    /// Returns the name of the subcommand, that is, the first positional argument.
    pub fn subcommand(&mut self) -> Result<Option<String>, Error> {
        if self.0.is_empty() {
            return Ok(None);
        }

        if let Some(s) = self.0[0].to_str() {
            if s.starts_with('-') {
                return Ok(None);
            }
        }

        self.0.remove(0)
            .into_string()
            .map_err(|_| Error::NonUtf8Argument)
            .map(Some)
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
    pub fn value_from_str<A, T>(&mut self, keys: A) -> Result<T, Error>
    where
        A: Into<Keys>,
        T: FromStr,
        <T as FromStr>::Err: Display,
    {
        self.value_from_fn(keys, FromStr::from_str)
    }

    /// Parses a key-value pair using a specified function.
    ///
    /// When a key-value pair is separated by a space, the algorithm
    /// will threat the next argument after the key as a value,
    /// even if it has a `-/--` prefix.
    /// So a key-value pair like `--key --value` is not an error.
    ///
    /// Must be used only once for each option.
    ///
    /// # Errors
    ///
    /// - When option is not present.
    /// - When key or value is not a UTF-8 string. Use [`value_from_os_str`] instead.
    /// - When value parsing failed.
    /// - When key-value pair is separated not by space or `=`.
    ///
    /// [`value_from_os_str`]: struct.Arguments.html#method.value_from_os_str
    pub fn value_from_fn<A: Into<Keys>, T, E: Display>(
        &mut self,
        keys: A,
        f: fn(&str) -> Result<T, E>,
    ) -> Result<T, Error> {
        let keys = keys.into();
        match self.opt_value_from_fn(keys, f) {
            Ok(Some(v)) => Ok(v),
            Ok(None) => Err(Error::MissingOption(keys)),
            Err(e) => Err(e),
        }
    }

    /// Parses an optional key-value pair using `FromStr` trait.
    ///
    /// This is a shorthand for `opt_value_from_fn("--key", FromStr::from_str)`
    pub fn opt_value_from_str<A, T>(&mut self, keys: A) -> Result<Option<T>, Error>
    where
        A: Into<Keys>,
        T: FromStr,
        <T as FromStr>::Err: Display,
    {
        self.opt_value_from_fn(keys, FromStr::from_str)
    }

    /// Parses an optional key-value pair using a specified function.
    ///
    /// The same as [`value_from_fn`], but returns `Ok(None)` when option is not present.
    ///
    /// [`value_from_fn`]: struct.Arguments.html#method.value_from_fn
    pub fn opt_value_from_fn<A: Into<Keys>, T, E: Display>(
        &mut self,
        keys: A,
        f: fn(&str) -> Result<T, E>,
    ) -> Result<Option<T>, Error> {
        self.opt_value_from_fn_impl(keys.into(), f)
    }

    #[inline(never)]
    fn opt_value_from_fn_impl<T, E: Display>(
        &mut self,
        keys: Keys,
        f: fn(&str) -> Result<T, E>,
    ) -> Result<Option<T>, Error> {
        match self.find_value(keys)? {
            Some((value, kind, idx)) => {
                match f(value) {
                    Ok(value) => {
                        // Remove only when all checks are passed.
                        self.0.remove(idx);
                        if kind == PairKind::TwoArguments {
                            self.0.remove(idx);
                        }

                        Ok(Some(value))
                    }
                    Err(e) => {
                        Err(Error::Utf8ArgumentParsingFailed {
                            value: value.to_string(),
                            cause: error_to_string(e),
                        })
                    }
                }
            }
            None => Ok(None),
        }
    }

    // The whole logic must be type-independent to prevent monomorphization.
    #[cfg(feature = "eq-separator")]
    #[inline(never)]
    fn find_value(
        &mut self,
        keys: Keys,
    ) -> Result<Option<(&str, PairKind, usize)>, Error> {
        if let Some((idx, key)) = self.index_of(keys) {
            // Parse a `--key value` pair.

            let value = match self.0.get(idx + 1) {
                Some(v) => v,
                None => return Err(Error::OptionWithoutAValue(key)),
            };

            let value = os_to_str(value)?;
            Ok(Some((value, PairKind::TwoArguments, idx)))
        } else if let Some((idx, key)) = self.index_of2(keys) {
            // Parse a `--key=value` pair.

            let value = &self.0[idx];

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

            Ok(Some((value, PairKind::SingleArgument, idx)))
        } else {
            Ok(None)
        }
    }

    // The whole logic must be type-independent to prevent monomorphization.
    #[cfg(not(feature = "eq-separator"))]
    #[inline(never)]
    fn find_value(
        &mut self,
        keys: Keys,
    ) -> Result<Option<(&str, PairKind, usize)>, Error> {
        if let Some((idx, key)) = self.index_of(keys) {
            // Parse a `--key value` pair.

            let value = match self.0.get(idx + 1) {
                Some(v) => v,
                None => return Err(Error::OptionWithoutAValue(key)),
            };

            let value = os_to_str(value)?;
            Ok(Some((value, PairKind::TwoArguments, idx)))
        } else {
            Ok(None)
        }
    }

    /// Parses multiple key-value pairs into the `Vec` using `FromStr` trait.
    ///
    /// This is a shorthand for `values_from_fn("--key", FromStr::from_str)`
    pub fn values_from_str<A, T>(&mut self, keys: A) -> Result<Vec<T>, Error>
        where
            A: Into<Keys>,
            T: FromStr,
            <T as FromStr>::Err: Display,
    {
        self.values_from_fn(keys, FromStr::from_str)
    }

    /// Parses multiple key-value pairs into the `Vec` using a specified function.
    ///
    /// This functions can be used to parse arguments like:<br>
    /// `--file /path1 --file /path2 --file /path3`<br>
    /// But not `--file /path1 /path2 /path3`.
    ///
    /// Arguments can also be separated: `--file /path1 --some-flag --file /path2`
    ///
    /// This method simply executes [`opt_value_from_fn`] multiple times.
    ///
    /// An empty `Vec` is not an error.
    ///
    /// [`opt_value_from_fn`]: struct.Arguments.html#method.opt_value_from_fn
    pub fn values_from_fn<A: Into<Keys>, T, E: Display>(
        &mut self,
        keys: A,
        f: fn(&str) -> Result<T, E>,
    ) -> Result<Vec<T>, Error> {
        let keys = keys.into();

        let mut values = Vec::new();
        loop {
            match self.opt_value_from_fn(keys, f) {
                Ok(Some(v)) => values.push(v),
                Ok(None) => break,
                Err(e) => return Err(e),
            }
        }

        Ok(values)
    }

    /// Parses a key-value pair using a specified function.
    ///
    /// Unlike [`value_from_fn`], parses `&OsStr` and not `&str`.
    ///
    /// Must be used only once for each option.
    ///
    /// # Errors
    ///
    /// - When option is not present.
    /// - When value parsing failed.
    /// - When key-value pair is separated not by space.
    ///   Only [`value_from_fn`] supports `=` separator.
    ///
    /// [`value_from_fn`]: struct.Arguments.html#method.value_from_fn
    pub fn value_from_os_str<A: Into<Keys>, T, E: Display>(
        &mut self,
        keys: A,
        f: fn(&OsStr) -> Result<T, E>,
    ) -> Result<T, Error> {
        let keys = keys.into();
        match self.opt_value_from_os_str(keys, f) {
            Ok(Some(v)) => Ok(v),
            Ok(None) => Err(Error::MissingOption(keys)),
            Err(e) => Err(e),
        }
    }

    /// Parses an optional key-value pair using a specified function.
    ///
    /// The same as `value_from_os_str`, but returns `Ok(None)` when option is not present.
    ///
    /// [`value_from_os_str`]: struct.Arguments.html#method.value_from_os_str
    pub fn opt_value_from_os_str<A: Into<Keys>, T, E: Display>(
        &mut self,
        keys: A,
        f: fn(&OsStr) -> Result<T, E>,
    ) -> Result<Option<T>, Error> {
        self.opt_value_from_os_str_impl(keys.into(), f)
    }

    #[inline(never)]
    fn opt_value_from_os_str_impl<T, E: Display>(
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

    /// Parses multiple key-value pairs into the `Vec` using a specified function.
    ///
    /// This method simply executes [`opt_value_from_os_str`] multiple times.
    ///
    /// Unlike [`values_from_fn`], parses `&OsStr` and not `&str`.
    ///
    /// An empty `Vec` is not an error.
    ///
    /// [`opt_value_from_os_str`]: struct.Arguments.html#method.opt_value_from_os_str
    /// [`values_from_fn`]: struct.Arguments.html#method.values_from_fn
    pub fn values_from_os_str<A: Into<Keys>, T, E: Display>(
        &mut self,
        keys: A,
        f: fn(&OsStr) -> Result<T, E>,
    ) -> Result<Vec<T>, Error> {
        let keys = keys.into();
        let mut values = Vec::new();
        loop {
            match self.opt_value_from_os_str(keys, f) {
                Ok(Some(v)) => values.push(v),
                Ok(None) => break,
                Err(e) => return Err(e),
            }
        }

        Ok(values)
    }

    #[inline(never)]
    fn index_of(&self, keys: Keys) -> Option<(usize, &'static str)> {
        // Do not unroll loop to save space, because it creates a bigger file.
        // Which is strange, since `index_of2` actually benefits from it.

        for key in &keys.0 {
            if !key.is_empty() {
                if let Some(i) = self.0.iter().position(|v| v == key) {
                    return Some((i, key));
                }
            }
        }

        None
    }

    #[cfg(feature = "eq-separator")]
    #[inline(never)]
    fn index_of2(&self, keys: Keys) -> Option<(usize, &'static str)> {
        // Loop unroll to save space.

        if !keys.first().is_empty() {
            if let Some(i) = self.0.iter().position(|v| starts_with_plus_eq(v, keys.first())) {
                return Some((i, keys.first()));
            }
        }

        if !keys.second().is_empty() {
            if let Some(i) = self.0.iter().position(|v| starts_with_plus_eq(v, keys.second())) {
                return Some((i, keys.second()));
            }
        }

        None
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
    /// - When argument is not a UTF-8 string. Use [`free_from_os_str`] instead.
    /// - When value parsing failed.
    ///
    /// [`value_from_os_str`]: struct.Arguments.html#method.value_from_os_str
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

        // This code produces 1.7KiB
        //
        // let mut args = Vec::new();
        // for arg in self.0 {
        //     let arg = os_to_str(arg.as_os_str())?.to_string();
        //     args.push(arg);
        // }

        // And this one is only 874B

        for arg in &self.0 {
            os_to_str(arg.as_os_str())?;
        }

        let args = self.0.iter().map(|a| a.to_str().unwrap().to_string()).collect();
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
    /// Use it instead of [`free`] if you do not expect any free arguments.
    ///
    /// [`free`]: struct.Arguments.html#method.free
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

#[cfg(feature = "eq-separator")]
#[inline(never)]
fn starts_with_plus_eq(text: &OsStr, prefix: &str) -> bool {
    if let Some(s) = text.to_str() {
        if s.get(0..prefix.len()) == Some(prefix) {
            if s.as_bytes().get(prefix.len()) == Some(&b'=') {
                return true;
            }
        }
    }

    false
}

#[cfg(feature = "eq-separator")]
#[inline]
fn ends_with(text: &str, c: u8) -> bool {
    if text.is_empty() {
        false
    } else {
        text.as_bytes()[text.len() - 1] == c
    }
}

#[inline]
fn os_to_str(text: &OsStr) -> Result<&str, Error> {
    text.to_str().ok_or_else(|| Error::NonUtf8Argument)
}


/// A keys container.
///
/// Should not be used directly.
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct Keys([&'static str; 2]);

impl Keys {
    #[inline]
    fn first(&self) -> &'static str {
        self.0[0]
    }

    #[inline]
    fn second(&self) -> &'static str {
        self.0[1]
    }
}

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
