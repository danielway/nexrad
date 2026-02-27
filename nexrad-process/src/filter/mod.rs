//! Filtering algorithms for sweep field data.
//!
//! Filters transform a [`SweepField`](nexrad_model::data::SweepField) by modifying gate values
//! or statuses based on various criteria such as value thresholds, spatial patterns, or
//! cross-product relationships.

mod clutter;
mod smoothing;
mod threshold;

pub use clutter::CorrelationCoefficientFilter;
pub use smoothing::{GaussianSmooth, MedianFilter};
pub use threshold::ThresholdFilter;
