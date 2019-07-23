# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
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

[Unreleased]: https://github.com/RazrFalcon/ttf-parser/compare/v0.1.0...HEAD
