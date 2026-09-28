# Contributing

Thanks for helping improve `binlayout`.

## Project scope

`binlayout` calculates sizes, alignments, field offsets, and padding for binary
structures. It does not read or write binary data. Keep host and target layouts
separate, use checked `u64` arithmetic for layout calculations, and preserve
the crate's `unsafe_code = "forbid"` lint.

## Toolchain

The minimum supported Rust version is 1.85, as declared in `Cargo.toml`. CI
checks Rust 1.85.0 and the current stable toolchain. Install both toolchains,
including the formatting and lint components used by CI, with:

```sh
rustup toolchain install 1.85.0
rustup toolchain install stable --component rustfmt --component clippy
```

## Tests and checks

Run the tests on the minimum supported toolchain, with and without the default
`std` feature:

```sh
cargo +1.85.0 test --locked
cargo +1.85.0 test --locked --no-default-features
```

CI also runs the test suite on stable in debug and release mode:

```sh
cargo +stable test --locked
cargo +stable test --locked --no-default-features
cargo +stable test --locked --release
```

Formatting and Clippy commands from CI:

```sh
cargo +stable fmt --check
cargo +stable clippy --locked --all-targets --all-features -- -D warnings
cargo +stable clippy --locked --all-targets --no-default-features -- -D warnings
```

CI also runs the examples, checks the package, and builds the API documentation:

```sh
cargo +stable run --locked --example header
cargo +stable run --locked --example elf
cargo +stable run --locked --example protocol
cargo +stable package --locked
RUSTDOCFLAGS="-D warnings" cargo +stable doc --locked --no-deps
```

## Optional cross-target checks

CI checks the library for WebAssembly and runs tests for 32-bit Linux. These
checks need extra targets; 32-bit tests also need a linker and multilib support.
They are separate from the basic local workflow.

```sh
rustup target add wasm32-unknown-unknown --toolchain stable
cargo +stable check --locked --lib --no-default-features --target wasm32-unknown-unknown

rustup target add i686-unknown-linux-gnu --toolchain stable
# On Ubuntu, install the same system support used by CI:
sudo apt-get update
sudo apt-get install -y gcc-multilib
cargo +stable test --locked --target i686-unknown-linux-gnu
cargo +stable test --locked --no-default-features --target i686-unknown-linux-gnu
```

## Issues and pull requests

Browse the [good first issue list][good-first-issue-list] for a suitable first
task.

[good-first-issue-list]: https://github.com/LunaStev/binlayout/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22

When a pull request should close an issue, include `Closes #<number>` in its
description. Use a plain `#<number>` reference when the issue should remain
open. Summarize the change and list the checks you ran in the pull request.
