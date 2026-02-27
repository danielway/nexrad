//! Storm cell detection algorithms.
//!
//! This module provides algorithms for identifying and characterizing storm
//! cells in radar data using connected-component analysis on reflectivity
//! fields.

mod cell;

pub use cell::{StormCell, StormCellBounds, StormCellDetector};
