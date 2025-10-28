//!
//! Primitive aliases matching the types referenced in the ICD.
//!

use zerocopy::big_endian;

pub type Code1 = u8;
pub type Code2 = big_endian::U16;
pub type Integer1 = u8;
pub type Integer2 = big_endian::U16;
pub type Integer4 = big_endian::U32;
pub type Real4 = big_endian::F32;
pub type ScaledInteger1 = u8;
pub type ScaledInteger2 = big_endian::U16;
pub type ScaledSInteger2 = big_endian::I16;
pub type SInteger2 = big_endian::I16;
