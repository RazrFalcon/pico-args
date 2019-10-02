# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

## [0.3.0] - 2019-09-23
### Added
- Required arguments support.
- `Error::MissingOption` when option is required but not present.

### Changed
- Rename `value_from_str` into `opt_value_from_str`.
- Rename `value_from_fn` into `opt_value_from_fn`.
- Rename `value_from_os_str` into `opt_value_from_os_str`.
- `value_from_str`, `value_from_fn` and `value_from_os_str` will return `T` and not `Option<T>`
  from now.

## [0.2.0] - 2019-07-26
### Added
- Non UTF-8 arguments support.
- `free_from_str`, `free_from_fn` and `free_from_os_str`.
- `value_from_os_str`.

### Changed
- `value_from_fn` allows any error type that implements `Display` now
  and not only `String`.
- `from_args` -> `from_vec`. And it accepts `Vec<OsString>` now.
- The `Error` enum.

### Fixed
- Do not panic while parsing non UTF-8 arguments.

[Unreleased]: https://github.com/RazrFalcon/pico-args/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/RazrFalcon/pico-args/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/RazrFalcon/pico-args/compare/v0.1.0...v0.2.0
