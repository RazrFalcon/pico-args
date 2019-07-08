## pico-args
[![Build Status](https://travis-ci.org/RazrFalcon/pico-args.svg?branch=master)](https://travis-ci.org/RazrFalcon/pico-args)
[![Crates.io](https://img.shields.io/crates/v/pico-args.svg)](https://crates.io/crates/pico-args)
[![Documentation](https://docs.rs/pico-args/badge.svg)](https://docs.rs/pico-args)

An ultra simple CLI arguments parser.

- Only flags, options and free arguments are supported.
- Arguments must be separated by a space. `=` isn't supported.
- No help generation.
- No combined flags (like `-vvv` or `-abc`).

### Alternatives

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

### License

MIT
