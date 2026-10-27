//! Connected-region detection on sweep fields.
//!
//! [`ThresholdRegionDetector`] finds connected regions of gates at or above one reflectivity
//! threshold using connected-component labeling, and reports each as a [`ThresholdRegion`].
//! The names describe the computation, not a meteorological classification.

mod region;

pub use region::{ThresholdRegion, ThresholdRegionDetector};
