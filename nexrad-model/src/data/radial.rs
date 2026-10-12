use crate::data::{CfpMomentData, MomentData};

#[cfg(feature = "chrono")]
use chrono::{DateTime, Utc};

#[cfg(feature = "uom")]
use uom::si::{angle::degree, f64::Angle};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A single radar ray composed of a series of gates. This represents a single azimuth angle and
/// elevation angle pair at a point in time and contains the Level II data (reflectivity, velocity,
/// and spectrum width) for each range gate in that ray. The range of the radar and gate interval
/// distance determines the resolution of the ray and the number of gates in the ray.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Radial {
    collection_timestamp: i64,

    azimuth_number: u16,
    azimuth_angle_degrees: f32,
    azimuth_spacing_degrees: f32,

    radial_status: RadialStatus,

    elevation_number: u8,
    elevation_angle_degrees: f32,

    reflectivity: Option<MomentData>,
    velocity: Option<MomentData>,
    spectrum_width: Option<MomentData>,
    differential_reflectivity: Option<MomentData>,
    differential_phase: Option<MomentData>,
    correlation_coefficient: Option<MomentData>,
    clutter_filter_power: Option<CfpMomentData>,
}

impl Radial {
    /// Start building a radial from its required identifying properties.
    ///
    /// All other properties take the defaults documented on [`RadialBuilder`].
    pub fn builder(
        azimuth_number: u16,
        azimuth_angle_degrees: f32,
        elevation_number: u8,
        elevation_angle_degrees: f32,
    ) -> RadialBuilder {
        RadialBuilder {
            radial: Radial {
                collection_timestamp: 0,
                azimuth_number,
                azimuth_angle_degrees,
                azimuth_spacing_degrees: 1.0,
                radial_status: RadialStatus::IntermediateRadialData,
                elevation_number,
                elevation_angle_degrees,
                reflectivity: None,
                velocity: None,
                spectrum_width: None,
                differential_reflectivity: None,
                differential_phase: None,
                correlation_coefficient: None,
                clutter_filter_power: None,
            },
        }
    }

    /// The collection timestamp in milliseconds since midnight Jan 1, 1970 (epoch/UNIX timestamp).
    pub fn collection_timestamp(&self) -> i64 {
        self.collection_timestamp
    }

    /// The collection time for this radial and its data.
    #[cfg(feature = "chrono")]
    pub fn collection_time(&self) -> Option<DateTime<Utc>> {
        DateTime::from_timestamp_millis(self.collection_timestamp)
    }

    /// The index number for this radial's azimuth in the elevation sweep, ranging up to 720
    /// depending on the azimuthal resolution.
    pub fn azimuth_number(&self) -> u16 {
        self.azimuth_number
    }

    /// Azimuth angle this radial's data was collected at in degrees.
    pub fn azimuth_angle_degrees(&self) -> f32 {
        self.azimuth_angle_degrees
    }

    /// Azimuth angle this radial's data was collected at.
    #[cfg(feature = "uom")]
    pub fn azimuth(&self) -> Angle {
        Angle::new::<degree>(self.azimuth_angle_degrees as f64)
    }

    /// Azimuthal distance between radials in the sweep in degrees.
    pub fn azimuth_spacing_degrees(&self) -> f32 {
        self.azimuth_spacing_degrees
    }

    /// Azimuthal distance between radials in the sweep.
    #[cfg(feature = "uom")]
    pub fn azimuth_spacing(&self) -> Angle {
        Angle::new::<degree>(self.azimuth_spacing_degrees as f64)
    }

    /// The radial's position in the sequence of radials making up a scan.
    pub fn radial_status(&self) -> RadialStatus {
        self.radial_status
    }

    /// The elevation number for this radial in the scan.
    pub fn elevation_number(&self) -> u8 {
        self.elevation_number
    }

    /// Elevation angle this radial's data was collected at in degrees.
    pub fn elevation_angle_degrees(&self) -> f32 {
        self.elevation_angle_degrees
    }

    /// Elevation angle this radial's data was collected at.
    #[cfg(feature = "uom")]
    pub fn elevation_angle(&self) -> Angle {
        Angle::new::<degree>(self.elevation_angle_degrees as f64)
    }

    /// Reflectivity data for this radial if available.
    pub fn reflectivity(&self) -> Option<&MomentData> {
        self.reflectivity.as_ref()
    }

    /// Velocity data for this radial if available.
    pub fn velocity(&self) -> Option<&MomentData> {
        self.velocity.as_ref()
    }

    /// Spectrum width data for this radial if available.
    pub fn spectrum_width(&self) -> Option<&MomentData> {
        self.spectrum_width.as_ref()
    }

    /// Differential reflectivity data for this radial if available.
    pub fn differential_reflectivity(&self) -> Option<&MomentData> {
        self.differential_reflectivity.as_ref()
    }

    /// Differential phase data for this radial if available.
    pub fn differential_phase(&self) -> Option<&MomentData> {
        self.differential_phase.as_ref()
    }

    /// Correlation coefficient data for this radial if available.
    pub fn correlation_coefficient(&self) -> Option<&MomentData> {
        self.correlation_coefficient.as_ref()
    }

    /// Clutter filter power (CFP) data for this radial if available.
    /// CFP represents the difference between clutter-filtered and unfiltered reflectivity.
    pub fn clutter_filter_power(&self) -> Option<&CfpMomentData> {
        self.clutter_filter_power.as_ref()
    }
}

/// Builder for [`Radial`], created with [`Radial::builder`].
///
/// The azimuth number, azimuth angle, elevation number, and elevation angle are required. All
/// other properties are optional and default to:
///
/// - `collection_timestamp`: `0`
/// - `azimuth_spacing_degrees`: `1.0`
/// - `radial_status`: [`RadialStatus::IntermediateRadialData`]
/// - all moment data (`reflectivity`, `velocity`, `spectrum_width`,
///   `differential_reflectivity`, `differential_phase`, `correlation_coefficient`,
///   `clutter_filter_power`): `None`
#[derive(Debug, Clone)]
#[must_use]
pub struct RadialBuilder {
    radial: Radial,
}

impl RadialBuilder {
    /// Set the collection timestamp in milliseconds since the UNIX epoch. Defaults to `0`.
    pub fn collection_timestamp(mut self, collection_timestamp: i64) -> Self {
        self.radial.collection_timestamp = collection_timestamp;
        self
    }

    /// Set the azimuthal distance between radials in the sweep in degrees. Defaults to `1.0`.
    pub fn azimuth_spacing_degrees(mut self, azimuth_spacing_degrees: f32) -> Self {
        self.radial.azimuth_spacing_degrees = azimuth_spacing_degrees;
        self
    }

    /// Set the radial's position in the sequence of radials making up a scan. Defaults to
    /// [`RadialStatus::IntermediateRadialData`].
    pub fn radial_status(mut self, radial_status: RadialStatus) -> Self {
        self.radial.radial_status = radial_status;
        self
    }

    /// Set the reflectivity data for this radial. Defaults to `None` (no reflectivity data).
    pub fn reflectivity(mut self, reflectivity: MomentData) -> Self {
        self.radial.reflectivity = Some(reflectivity);
        self
    }

    /// Set the velocity data for this radial. Defaults to `None` (no velocity data).
    pub fn velocity(mut self, velocity: MomentData) -> Self {
        self.radial.velocity = Some(velocity);
        self
    }

    /// Set the spectrum width data for this radial. Defaults to `None` (no spectrum width data).
    pub fn spectrum_width(mut self, spectrum_width: MomentData) -> Self {
        self.radial.spectrum_width = Some(spectrum_width);
        self
    }

    /// Set the differential reflectivity data for this radial. Defaults to `None` (no differential reflectivity data).
    pub fn differential_reflectivity(mut self, differential_reflectivity: MomentData) -> Self {
        self.radial.differential_reflectivity = Some(differential_reflectivity);
        self
    }

    /// Set the differential phase data for this radial. Defaults to `None` (no differential phase data).
    pub fn differential_phase(mut self, differential_phase: MomentData) -> Self {
        self.radial.differential_phase = Some(differential_phase);
        self
    }

    /// Set the correlation coefficient data for this radial. Defaults to `None` (no correlation coefficient data).
    pub fn correlation_coefficient(mut self, correlation_coefficient: MomentData) -> Self {
        self.radial.correlation_coefficient = Some(correlation_coefficient);
        self
    }

    /// Set the clutter filter power (CFP) data for this radial. Defaults to `None` (no CFP
    /// data).
    pub fn clutter_filter_power(mut self, clutter_filter_power: CfpMomentData) -> Self {
        self.radial.clutter_filter_power = Some(clutter_filter_power);
        self
    }

    /// Build the [`Radial`].
    #[must_use]
    pub fn build(self) -> Radial {
        self.radial
    }
}

/// Describe a radial's position within the sequence of radials comprising a scan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum RadialStatus {
    /// First radial of an elevation sweep (not the first sweep in the scan).
    ElevationStart,
    /// A radial within an elevation sweep (not the first or last radial).
    IntermediateRadialData,
    /// Last radial of an elevation sweep.
    ElevationEnd,
    /// First radial of the first elevation sweep in a scan.
    ScanStart,
    /// Last radial of the last elevation sweep in a scan.
    ScanEnd,
    /// Start of new elevation which is the last in the VCP.
    ElevationStartVcpFinal,
    /// An unrecognized radial status value.
    Unknown(u8),
}
