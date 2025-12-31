//!
//! Primitive aliases matching the types referenced in the ICD.
//!

use std::cmp::Ordering;
use std::fmt;
use std::ops::{BitAnd, Shr};
use zerocopy::{big_endian, FromBytes, Immutable, KnownLayout};

pub type Code1 = u8;
pub type Integer1 = u8;
pub type ScaledInteger1 = u8;

/// Big-endian unsigned 16-bit integer (Code2 in ICD).
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, FromBytes, Immutable, KnownLayout)]
pub struct Code2(big_endian::U16);

impl Code2 {
    pub fn new(value: u16) -> Self {
        Self(big_endian::U16::new(value))
    }

    pub fn get(self) -> u16 {
        self.0.get()
    }
}

impl fmt::Debug for Code2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.get().fmt(f)
    }
}

impl fmt::Display for Code2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.get().fmt(f)
    }
}

impl PartialEq<u16> for Code2 {
    fn eq(&self, other: &u16) -> bool {
        self.0.get() == *other
    }
}

impl PartialOrd<u16> for Code2 {
    fn partial_cmp(&self, other: &u16) -> Option<Ordering> {
        self.0.get().partial_cmp(other)
    }
}

impl BitAnd<u16> for Code2 {
    type Output = u16;
    fn bitand(self, rhs: u16) -> u16 {
        self.0.get() & rhs
    }
}

impl Shr<i32> for Code2 {
    type Output = u16;
    fn shr(self, rhs: i32) -> u16 {
        self.0.get() >> rhs
    }
}

impl From<u16> for Code2 {
    fn from(value: u16) -> Self {
        Self::new(value)
    }
}

/// Big-endian unsigned 16-bit integer (Integer2 in ICD).
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, FromBytes, Immutable, KnownLayout)]
pub struct Integer2(big_endian::U16);

impl Integer2 {
    pub fn new(value: u16) -> Self {
        Self(big_endian::U16::new(value))
    }

    pub fn get(self) -> u16 {
        self.0.get()
    }
}

impl fmt::Debug for Integer2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.get().fmt(f)
    }
}

impl fmt::Display for Integer2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.get().fmt(f)
    }
}

impl PartialEq<u16> for Integer2 {
    fn eq(&self, other: &u16) -> bool {
        self.0.get() == *other
    }
}

impl PartialOrd<u16> for Integer2 {
    fn partial_cmp(&self, other: &u16) -> Option<Ordering> {
        self.0.get().partial_cmp(other)
    }
}

impl From<u16> for Integer2 {
    fn from(value: u16) -> Self {
        Self::new(value)
    }
}

/// Big-endian unsigned 32-bit integer (Integer4 in ICD).
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, FromBytes, Immutable, KnownLayout)]
pub struct Integer4(big_endian::U32);

impl Integer4 {
    pub fn new(value: u32) -> Self {
        Self(big_endian::U32::new(value))
    }

    pub fn get(self) -> u32 {
        self.0.get()
    }
}

impl fmt::Debug for Integer4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.get().fmt(f)
    }
}

impl fmt::Display for Integer4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.get().fmt(f)
    }
}

impl PartialEq<u32> for Integer4 {
    fn eq(&self, other: &u32) -> bool {
        self.0.get() == *other
    }
}

impl PartialOrd<u32> for Integer4 {
    fn partial_cmp(&self, other: &u32) -> Option<Ordering> {
        self.0.get().partial_cmp(other)
    }
}

impl From<u32> for Integer4 {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

/// Big-endian 32-bit float (Real4 in ICD).
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, FromBytes, Immutable, KnownLayout)]
pub struct Real4(big_endian::F32);

impl Real4 {
    pub fn new(value: f32) -> Self {
        Self(big_endian::F32::new(value))
    }

    pub fn get(self) -> f32 {
        self.0.get()
    }
}

impl fmt::Debug for Real4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.get().fmt(f)
    }
}

impl fmt::Display for Real4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.get().fmt(f)
    }
}

impl PartialEq<f32> for Real4 {
    fn eq(&self, other: &f32) -> bool {
        self.0.get() == *other
    }
}

impl PartialOrd<f32> for Real4 {
    fn partial_cmp(&self, other: &f32) -> Option<Ordering> {
        self.0.get().partial_cmp(other)
    }
}

impl From<f32> for Real4 {
    fn from(value: f32) -> Self {
        Self::new(value)
    }
}

/// Big-endian unsigned 16-bit integer (ScaledInteger2 in ICD).
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, FromBytes, Immutable, KnownLayout)]
pub struct ScaledInteger2(big_endian::U16);

impl ScaledInteger2 {
    pub fn new(value: u16) -> Self {
        Self(big_endian::U16::new(value))
    }

    pub fn get(self) -> u16 {
        self.0.get()
    }
}

impl fmt::Debug for ScaledInteger2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.get().fmt(f)
    }
}

impl fmt::Display for ScaledInteger2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.get().fmt(f)
    }
}

impl PartialEq<u16> for ScaledInteger2 {
    fn eq(&self, other: &u16) -> bool {
        self.0.get() == *other
    }
}

impl PartialOrd<u16> for ScaledInteger2 {
    fn partial_cmp(&self, other: &u16) -> Option<Ordering> {
        self.0.get().partial_cmp(other)
    }
}

impl From<u16> for ScaledInteger2 {
    fn from(value: u16) -> Self {
        Self::new(value)
    }
}

/// Big-endian signed 16-bit integer (ScaledSInteger2 in ICD).
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, FromBytes, Immutable, KnownLayout)]
pub struct ScaledSInteger2(big_endian::I16);

impl ScaledSInteger2 {
    pub fn new(value: i16) -> Self {
        Self(big_endian::I16::new(value))
    }

    pub fn get(self) -> i16 {
        self.0.get()
    }
}

impl fmt::Debug for ScaledSInteger2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.get().fmt(f)
    }
}

impl fmt::Display for ScaledSInteger2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.get().fmt(f)
    }
}

impl PartialEq<i16> for ScaledSInteger2 {
    fn eq(&self, other: &i16) -> bool {
        self.0.get() == *other
    }
}

impl PartialOrd<i16> for ScaledSInteger2 {
    fn partial_cmp(&self, other: &i16) -> Option<Ordering> {
        self.0.get().partial_cmp(other)
    }
}

impl From<i16> for ScaledSInteger2 {
    fn from(value: i16) -> Self {
        Self::new(value)
    }
}

/// Big-endian signed 16-bit integer (SInteger2 in ICD).
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, FromBytes, Immutable, KnownLayout)]
pub struct SInteger2(big_endian::I16);

impl SInteger2 {
    pub fn new(value: i16) -> Self {
        Self(big_endian::I16::new(value))
    }

    pub fn get(self) -> i16 {
        self.0.get()
    }
}

impl fmt::Debug for SInteger2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.get().fmt(f)
    }
}

impl fmt::Display for SInteger2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.get().fmt(f)
    }
}

impl PartialEq<i16> for SInteger2 {
    fn eq(&self, other: &i16) -> bool {
        self.0.get() == *other
    }
}

impl PartialOrd<i16> for SInteger2 {
    fn partial_cmp(&self, other: &i16) -> Option<Ordering> {
        self.0.get().partial_cmp(other)
    }
}

impl From<i16> for SInteger2 {
    fn from(value: i16) -> Self {
        Self::new(value)
    }
}
