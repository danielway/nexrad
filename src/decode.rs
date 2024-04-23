//!
//! TODO
//!

use crate::model::Archive2Header;
use crate::result::Result;
use bincode::{DefaultOptions, Options};
use serde::de::DeserializeOwned;
use std::io::Read;

pub fn decode_archive2_header<R: Read>(reader: &mut R) -> Result<Archive2Header> {
    deserialize(reader)
}

/// Attempts to deserialize some struct from the provided binary reader.
fn deserialize<R: Read, S: DeserializeOwned>(reader: &mut R) -> Result<S> {
    Ok(DefaultOptions::new()
        .with_fixint_encoding()
        .with_big_endian()
        .deserialize_from(reader.by_ref())?)
}
