use crate::messages::clutter_censor_zones::raw;
use std::borrow::Cow;
use std::fmt::{self, Debug};

/// A single clutter censor zone override region.
///
/// Each region defines a range, azimuth, and elevation zone along with an
/// [`OperatorSelectCode`] that controls how clutter filtering is applied within the zone.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Region<'a> {
    raw: Cow<'a, raw::Region>,
}

impl Debug for Region<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.raw.fmt(f)
    }
}

impl<'a> Region<'a> {
    /// Wrap a region borrowed from the input.
    pub(crate) fn from_raw(raw: &'a raw::Region) -> Self {
        Self {
            raw: Cow::Borrowed(raw),
        }
    }

    /// Start range in km (0-511).
    pub fn start_range_km(&self) -> u16 {
        self.raw.start_range.get()
    }

    /// Stop range in km (0-511).
    pub fn stop_range_km(&self) -> u16 {
        self.raw.stop_range.get()
    }

    /// Start azimuth in degrees (0-360).
    pub fn start_azimuth_degrees(&self) -> u16 {
        self.raw.start_azimuth.get()
    }

    /// Stop azimuth in degrees (0-360).
    pub fn stop_azimuth_degrees(&self) -> u16 {
        self.raw.stop_azimuth.get()
    }

    /// Elevation segment number (1-5).
    pub fn elevation_segment_number(&self) -> u16 {
        self.raw.elevation_segment_number.get()
    }

    /// Operator select code controlling clutter filtering behavior within this region.
    pub fn operator_select_code(&self) -> OperatorSelectCode {
        match self.operator_select_code_raw() {
            0 => OperatorSelectCode::BypassFilterForced,
            1 => OperatorSelectCode::BypassMapInControl,
            2 => OperatorSelectCode::ClutterFilteringForced,
            other => OperatorSelectCode::Unknown(other),
        }
    }

    /// The raw operator select code, as it appears on the wire. See
    /// [`operator_select_code()`](Self::operator_select_code) for the decoded value.
    pub fn operator_select_code_raw(&self) -> u16 {
        self.raw.operator_select_code.get()
    }

    /// Convert this region to an owned version with `'static` lifetime.
    pub fn into_owned(self) -> Region<'static> {
        Region {
            raw: Cow::Owned(self.raw.into_owned()),
        }
    }
}

/// Operator select code for a clutter censor zone override region, controlling how clutter
/// filtering is applied within the region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum OperatorSelectCode {
    /// Code 0: the bypass filter is forced.
    BypassFilterForced,

    /// Code 1: the bypass map is in control.
    BypassMapInControl,

    /// Code 2: clutter filtering is forced.
    ClutterFilteringForced,

    /// A code not defined by the ICD, carrying the raw value.
    Unknown(u16),
}
