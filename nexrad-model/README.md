# NEXRAD Model

[![Crates.io](https://img.shields.io/crates/v/nexrad-model)](https://crates.io/crates/nexrad-model)
[![Docs.rs](https://docs.rs/nexrad-model/badge.svg)](https://docs.rs/nexrad-model)
[![Rust CI](https://github.com/danielway/nexrad/actions/workflows/rust_ci.yml/badge.svg?branch=master)](https://github.com/danielway/nexrad/actions/workflows/rust_ci.yml)
[![Rust CD](https://github.com/danielway/nexrad/actions/workflows/rust_cd.yml/badge.svg)](https://github.com/danielway/nexrad/actions/workflows/rust_cd.yml)

A common model for representing NEXRAD weather radar data. Provides an ergonomic API which is documented for an audience
who is not necessarily familiar with the NOAA Archive II format.

## Features

- `uom`: Use the `uom` crate for type-safe units of measure.
- `serde`: Implement `serde::Serialize` and `serde::Deserialize` for all models.
- `chrono`: Use the `chrono` crate for date and time types.