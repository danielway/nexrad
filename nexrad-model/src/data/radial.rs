use crate::data::{MomentData, Product};
use crate::result::{Error, Result};

#[cfg(feature = "chrono")]
use chrono::{DateTime, Utc};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A single radar ray composed of a series of gates. This represents a single azimuth angle and
/// elevation angle pair at a point in time and contains the Level II data (reflectivity, velocity,
/// and spectrum width) for each range gate in that ray. The range of the radar and gate interval
/// distance determines the resolution of the ray and the number of gates in the ray.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Radial {
    collection_timestamp_millis: i64,

    azimuth_number: u16,
    azimuth_angle_degrees: f32,
    azimuth_spacing_degrees: f32,

    radial_status: RadialStatus,

    elevation_number: u8,
    elevation_angle_degrees: f32,

    nyquist_velocity_mps: Option<f32>,
    unambiguous_range_km: Option<f32>,

    reflectivity: Option<MomentData>,
    velocity: Option<MomentData>,
    spectrum_width: Option<MomentData>,
    differential_reflectivity: Option<MomentData>,
    differential_phase: Option<MomentData>,
    correlation_coefficient: Option<MomentData>,
    clutter_filter_power: Option<MomentData>,
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
                collection_timestamp_millis: 0,
                azimuth_number,
                azimuth_angle_degrees,
                azimuth_spacing_degrees: 1.0,
                radial_status: RadialStatus::IntermediateRadialData,
                elevation_number,
                elevation_angle_degrees,
                nyquist_velocity_mps: None,
                unambiguous_range_km: None,
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

    /// The collection timestamp in milliseconds since the Unix epoch, UTC.
    pub fn collection_timestamp_millis(&self) -> i64 {
        self.collection_timestamp_millis
    }

    /// The collection time for this radial and its data.
    #[cfg(feature = "chrono")]
    pub fn collection_time(&self) -> Option<DateTime<Utc>> {
        DateTime::from_timestamp_millis(self.collection_timestamp_millis)
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

    /// Azimuthal distance between radials in the sweep in degrees.
    pub fn azimuth_spacing_degrees(&self) -> f32 {
        self.azimuth_spacing_degrees
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

    /// The Nyquist velocity in meters per second, if the source carried it.
    ///
    /// This is the largest radial velocity that can be measured unambiguously with the pulse
    /// repetition frequency (PRF) used on this radial; faster motion aliases into the
    /// `-nyquist..nyquist` interval. It can differ between radials of one sweep because Doppler
    /// PRF sectors vary by azimuth, so dealiasing must use the per-radial value.
    pub fn nyquist_velocity_mps(&self) -> Option<f32> {
        self.nyquist_velocity_mps
    }

    /// The unambiguous range in kilometers, if the source carried it.
    ///
    /// This is the maximum range for the pulse repetition frequency used on this radial before
    /// echoes from beyond it (second-trip echoes) fold back to a shorter apparent range.
    pub fn unambiguous_range_km(&self) -> Option<f32> {
        self.unambiguous_range_km
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
    pub fn clutter_filter_power(&self) -> Option<&MomentData> {
        self.clutter_filter_power.as_ref()
    }

    /// Moment data for the given product in this radial, if available.
    ///
    /// This is the uniform accessor for all products; the named getters such as
    /// [`reflectivity`](Self::reflectivity) are equivalent shorthands.
    pub fn moment(&self, product: Product) -> Option<&MomentData> {
        match product {
            Product::Reflectivity => self.reflectivity(),
            Product::Velocity => self.velocity(),
            Product::SpectrumWidth => self.spectrum_width(),
            Product::DifferentialReflectivity => self.differential_reflectivity(),
            Product::DifferentialPhase => self.differential_phase(),
            Product::CorrelationCoefficient => self.correlation_coefficient(),
            Product::ClutterFilterPower => self.clutter_filter_power(),
        }
    }
}

/// Builder for [`Radial`], created with [`Radial::builder`].
///
/// The azimuth number, azimuth angle, elevation number, and elevation angle are required. All
/// other properties are optional and default to:
///
/// - `collection_timestamp_millis`: `0`
/// - `azimuth_spacing_degrees`: `1.0`
/// - `radial_status`: [`RadialStatus::IntermediateRadialData`]
/// - `nyquist_velocity_mps` and `unambiguous_range_km`: `None`
/// - moment data for every product: `None` (set with [`moment`](Self::moment))
///
/// [`build`](Self::build) returns [`Error::InvalidGeometry`] unless the azimuth angle is finite
/// and in `0.0..360.0`, the elevation angle is finite, and the azimuth spacing is finite and
/// greater than zero, and unless the Nyquist velocity and unambiguous range, when set, are finite
/// and greater than zero.
#[derive(Debug, Clone)]
#[must_use]
pub struct RadialBuilder {
    radial: Radial,
}

impl RadialBuilder {
    /// Set the collection timestamp in milliseconds since the Unix epoch, UTC. Defaults to `0`.
    pub fn collection_timestamp_millis(mut self, collection_timestamp_millis: i64) -> Self {
        self.radial.collection_timestamp_millis = collection_timestamp_millis;
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

    /// Set the Nyquist velocity in meters per second: the largest unambiguous radial velocity
    /// for the pulse repetition frequency used on this radial. Defaults to `None`. See
    /// [`Radial::nyquist_velocity_mps`].
    pub fn nyquist_velocity_mps(mut self, nyquist_velocity_mps: f32) -> Self {
        self.radial.nyquist_velocity_mps = Some(nyquist_velocity_mps);
        self
    }

    /// Set the unambiguous range in kilometers: the maximum range before second-trip echoes
    /// fold. Defaults to `None`. See [`Radial::unambiguous_range_km`].
    pub fn unambiguous_range_km(mut self, unambiguous_range_km: f32) -> Self {
        self.radial.unambiguous_range_km = Some(unambiguous_range_km);
        self
    }

    /// Set the moment data for this radial, stored by its [`MomentData::product`]. Setting
    /// moment data for a product that is already set replaces it. Every moment defaults to
    /// `None` (no data for that product).
    pub fn moment(mut self, data: MomentData) -> Self {
        let slot = match data.product() {
            Product::Reflectivity => &mut self.radial.reflectivity,
            Product::Velocity => &mut self.radial.velocity,
            Product::SpectrumWidth => &mut self.radial.spectrum_width,
            Product::DifferentialReflectivity => &mut self.radial.differential_reflectivity,
            Product::DifferentialPhase => &mut self.radial.differential_phase,
            Product::CorrelationCoefficient => &mut self.radial.correlation_coefficient,
            Product::ClutterFilterPower => &mut self.radial.clutter_filter_power,
        };
        *slot = Some(data);
        self
    }

    /// Build the [`Radial`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidGeometry`] if any rule listed on [`RadialBuilder`] is broken.
    pub fn build(self) -> Result<Radial> {
        let radial = &self.radial;
        if !(0.0..360.0).contains(&radial.azimuth_angle_degrees) {
            return Err(Error::InvalidGeometry {
                parameter: "azimuth_angle_degrees",
                reason: "must be finite and in 0.0..360.0",
            });
        }
        if !radial.elevation_angle_degrees.is_finite() {
            return Err(Error::InvalidGeometry {
                parameter: "elevation_angle_degrees",
                reason: "must be finite",
            });
        }
        if !(radial.azimuth_spacing_degrees.is_finite() && radial.azimuth_spacing_degrees > 0.0) {
            return Err(Error::InvalidGeometry {
                parameter: "azimuth_spacing_degrees",
                reason: "must be finite and greater than zero",
            });
        }
        if let Some(nyquist) = radial.nyquist_velocity_mps {
            if !(nyquist.is_finite() && nyquist > 0.0) {
                return Err(Error::InvalidGeometry {
                    parameter: "nyquist_velocity_mps",
                    reason: "must be finite and greater than zero",
                });
            }
        }
        if let Some(range) = radial.unambiguous_range_km {
            if !(range.is_finite() && range > 0.0) {
                return Err(Error::InvalidGeometry {
                    parameter: "unambiguous_range_km",
                    reason: "must be finite and greater than zero",
                });
            }
        }
        Ok(self.radial)
    }
}

/// Deserializes through [`Radial::builder`], so the geometry is validated and each moment must
/// be stored under the key for its own product.
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Radial {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let repr = RadialRepr::deserialize(deserializer)?;
        let mut builder = Radial::builder(
            repr.azimuth_number,
            repr.azimuth_angle_degrees,
            repr.elevation_number,
            repr.elevation_angle_degrees,
        )
        .collection_timestamp_millis(repr.collection_timestamp_millis)
        .azimuth_spacing_degrees(repr.azimuth_spacing_degrees)
        .radial_status(repr.radial_status);
        if let Some(nyquist_velocity_mps) = repr.nyquist_velocity_mps {
            builder = builder.nyquist_velocity_mps(nyquist_velocity_mps);
        }
        if let Some(unambiguous_range_km) = repr.unambiguous_range_km {
            builder = builder.unambiguous_range_km(unambiguous_range_km);
        }
        let moments = [
            (Product::Reflectivity, repr.reflectivity),
            (Product::Velocity, repr.velocity),
            (Product::SpectrumWidth, repr.spectrum_width),
            (
                Product::DifferentialReflectivity,
                repr.differential_reflectivity,
            ),
            (Product::DifferentialPhase, repr.differential_phase),
            (
                Product::CorrelationCoefficient,
                repr.correlation_coefficient,
            ),
            (Product::ClutterFilterPower, repr.clutter_filter_power),
        ];
        for (product, moment) in moments {
            if let Some(moment) = moment {
                if moment.product() != product {
                    return Err(serde::de::Error::custom(format_args!(
                        "moment data for {:?} stored under the key for {product:?}",
                        moment.product()
                    )));
                }
                builder = builder.moment(moment);
            }
        }
        builder.build().map_err(serde::de::Error::custom)
    }
}

/// The serialized form of [`Radial`], validated before it becomes one.
#[cfg(feature = "serde")]
#[derive(Deserialize)]
#[serde(rename = "Radial")]
struct RadialRepr {
    collection_timestamp_millis: i64,
    azimuth_number: u16,
    azimuth_angle_degrees: f32,
    azimuth_spacing_degrees: f32,
    radial_status: RadialStatus,
    elevation_number: u8,
    elevation_angle_degrees: f32,
    #[serde(default)]
    nyquist_velocity_mps: Option<f32>,
    #[serde(default)]
    unambiguous_range_km: Option<f32>,
    #[serde(default)]
    reflectivity: Option<MomentData>,
    #[serde(default)]
    velocity: Option<MomentData>,
    #[serde(default)]
    spectrum_width: Option<MomentData>,
    #[serde(default)]
    differential_reflectivity: Option<MomentData>,
    #[serde(default)]
    differential_phase: Option<MomentData>,
    #[serde(default)]
    correlation_coefficient: Option<MomentData>,
    #[serde(default)]
    clutter_filter_power: Option<MomentData>,
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
