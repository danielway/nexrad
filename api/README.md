# Public API baselines

Each `<crate>.txt` file in this directory is the public API of one published library crate
(`nexrad-model`, `nexrad-decode`, `nexrad-data`, `nexrad-process`, `nexrad-render` and `nexrad`)
with all features enabled, as listed by
[`cargo public-api`](https://github.com/cargo-public-api/cargo-public-api) with `--simplified`.
The files are committed so that every change to a crate's public API shows up as a diff in
review. CI regenerates them and fails if any file is out of date.

To regenerate the baselines after an intentional API change, install the tool with
`cargo install cargo-public-api --locked` and run `cargo run -p xtask -- public-api`; use
`cargo run -p xtask -- public-api --check` to compare without writing. Listing a public API
requires rustdoc JSON, so the command runs with the pinned nightly toolchain named in
`xtask/src/public_api.rs` (install it with `rustup toolchain install <toolchain>`); keep it in
sync with the `public-api` job in `.github/workflows/ci.yml`.
