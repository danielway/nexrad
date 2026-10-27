//! Connected-region detection on sweep fields.
//!
//! [`ThresholdRegionDetector`](crate::detection::ThresholdRegionDetector) finds connected
//! regions of gates at or above one reflectivity threshold using connected-component labeling,
//! and reports each as a [`ThresholdRegion`](crate::detection::ThresholdRegion). The names
//! describe the computation, not a meteorological classification.

mod region;

pub use region::{ThresholdRegion, ThresholdRegionDetector};
