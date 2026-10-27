use crate::messages::digital_radar_data::RadialStatus;
use crate::messages::digital_radar_data_legacy::raw;
use crate::messages::VcpNumber;
use crate::result::Result;
use crate::segmented_slice_reader::SegmentedSliceReader;
use crate::util::get_datetime;
use chrono::{DateTime, Duration, Utc};
use std::borrow::Cow;
use std::fmt::Debug;

/// A decoded Message Type 1 "Digital Radar Data" radial.
///
/// This is the legacy radar data format used from the original WSR-88D deployment
/// (1991) through the transition to Message Type 31 at Build 10.0 (March 2008).
/// Each message represents a single radial containing up to three base data
/// moments: reflectivity, velocity, and spectrum width.
///
/// Key differences from Message Type 31:
/// - Fixed 2432-byte frame size (vs. variable-length)
/// - Gate data is 1 byte per gate (vs. 1-2 bytes with configurable resolution)
/// - No dual-polarization moments (ZDR, PHI, RHO, CFP)
/// - No data block pointer table — moments are at fixed positions
/// - Reflectivity at 1 km gate spacing, Doppler at 250 m spacing
///
/// # Gate Value Encoding
///
/// All three moments use the same 1-byte encoding:
/// - `0` = Below Threshold (no detectable signal)
/// - `1` = Range Folded (ambiguous echo)
/// - `2..=255` = Scaled physical value (see moment-specific decoding)
///
/// Reflectivity: `dBZ = (value - 2) / 2.0 - 32.0` (range: -32.0 to 94.5 dBZ)
///
/// Velocity (resolution 2, 0.5 m/s): `m/s = (value - 2) * 0.5 - 63.5`
///
/// Velocity (resolution 4, 1.0 m/s): `m/s = (value - 2) * 1.0 - 127.0`
///
/// Spectrum Width: `m/s = (value - 2) * 0.5 - 63.5`
#[derive(Clone, PartialEq)]
pub struct Message<'a> {
    header: Cow<'a, raw::Header>,
    reflectivity_gates: Option<Cow<'a, [u8]>>,
    velocity_gates: Option<Cow<'a, [u8]>>,
    spectrum_width_gates: Option<Cow<'a, [u8]>>,
}

impl<'a> Message<'a> {
    pub(crate) fn parse(reader: &mut SegmentedSliceReader<'a, '_>) -> Result<Self> {
        let header = reader.take_ref::<raw::Header>()?;

        let surveillance_gate_count = header.surveillance_gate_count.get() as usize;
        let doppler_gate_count = header.doppler_gate_count.get() as usize;
        let ref_ptr = header.reflectivity_pointer.get() as usize;
        let vel_ptr = header.velocity_pointer.get() as usize;
        let sw_ptr = header.spectrum_width_pointer.get() as usize;

        // The header is 100 bytes. Gate data starts at byte 100 of the message body.
        // Pointers are byte offsets from the start of the message body.
        let header_size = size_of::<raw::Header>();

        let reflectivity_gates =
            if ref_ptr > 0 && surveillance_gate_count > 0 && ref_ptr >= header_size {
                // Skip any gap between header end and reflectivity data start
                let gap = ref_ptr - header_size;
                if gap > 0 {
                    reader.advance(gap);
                }
                let gates = reader.take_slice::<u8>(surveillance_gate_count)?;
                Some(Cow::Borrowed(gates))
            } else {
                None
            };

        let velocity_gates = if vel_ptr > 0 && doppler_gate_count > 0 {
            // Calculate current position and skip to velocity data
            let current_pos = if ref_ptr > 0 && surveillance_gate_count > 0 {
                ref_ptr + surveillance_gate_count
            } else {
                header_size
            };
            let gap = vel_ptr.saturating_sub(current_pos);
            if gap > 0 {
                reader.advance(gap);
            }
            let gates = reader.take_slice::<u8>(doppler_gate_count)?;
            Some(Cow::Borrowed(gates))
        } else {
            None
        };

        let spectrum_width_gates = if sw_ptr > 0 && doppler_gate_count > 0 {
            // Calculate current position and skip to spectrum width data
            let current_pos = if vel_ptr > 0 && doppler_gate_count > 0 {
                vel_ptr + doppler_gate_count
            } else if ref_ptr > 0 && surveillance_gate_count > 0 {
                ref_ptr + surveillance_gate_count
            } else {
                header_size
            };
            let gap = sw_ptr.saturating_sub(current_pos);
            if gap > 0 {
                reader.advance(gap);
            }
            let gates = reader.take_slice::<u8>(doppler_gate_count)?;
            Some(Cow::Borrowed(gates))
        } else {
            None
        };

        Ok(Self {
            header: Cow::Borrowed(header),
            reflectivity_gates,
            velocity_gates,
            spectrum_width_gates,
        })
    }

    /// Collection date and time in UTC.
    pub fn date_time(&self) -> Option<DateTime<Utc>> {
        get_datetime(
            self.header.modified_julian_date.get(),
            Duration::milliseconds(self.header.collection_time.get() as i64),
        )
    }

    /// Azimuth angle in degrees (0.0 to 360.0).
    pub fn azimuth_angle_degrees(&self) -> f32 {
        self.header.azimuth_angle.get() as f32 * 180.0 / 32768.0
    }

    /// Azimuth number within the current elevation (1-indexed).
    pub fn azimuth_number(&self) -> u16 {
        self.header.azimuth_number.get()
    }

    /// Elevation angle in degrees.
    pub fn elevation_angle_degrees(&self) -> f32 {
        self.header.elevation_angle.get() as f32 * 180.0 / 32768.0
    }

    /// Elevation number within the volume scan (1-indexed).
    ///
    /// The ICD range is 1 to 25. The wire field is two bytes wide; a value that does not fit in
    /// a `u8` (only possible in malformed data) is reported as `u8::MAX`.
    pub fn elevation_number(&self) -> u8 {
        u8::try_from(self.header.elevation_number.get()).unwrap_or(u8::MAX)
    }

    /// Radial status indicator (raw value). See [`radial_status()`](Self::radial_status) for the
    /// decoded value.
    ///
    /// - 0 = Start of new elevation
    /// - 1 = Intermediate radial
    /// - 2 = End of elevation
    /// - 3 = Beginning of volume scan
    /// - 4 = End of volume scan
    pub fn radial_status_raw(&self) -> u16 {
        self.header.radial_status.get()
    }

    /// The radial's status within the larger scan, using the same code mapping as the Message
    /// Type 31 header. A value that does not fit in a `u8` (only possible in malformed data) is
    /// reported as [`RadialStatus::Unknown(u8::MAX)`](RadialStatus::Unknown).
    pub fn radial_status(&self) -> RadialStatus {
        RadialStatus::from_number(u8::try_from(self.radial_status_raw()).unwrap_or(u8::MAX))
    }

    /// Volume Coverage Pattern number (raw value). See [`vcp_number()`](Self::vcp_number) for the
    /// decoded value.
    pub fn vcp_number_raw(&self) -> u16 {
        self.header.vcp_number.get()
    }

    /// Volume Coverage Pattern number.
    pub fn vcp_number(&self) -> VcpNumber {
        VcpNumber::new(self.vcp_number_raw())
    }

    /// Unambiguous range in kilometers.
    pub fn unambiguous_range_km(&self) -> f32 {
        self.header.unambiguous_range.get() as f32 / 10.0
    }

    /// Nyquist velocity in hundredths of a m/s (raw scaled value). See
    /// [`nyquist_velocity_mps()`](Self::nyquist_velocity_mps) for the decoded value.
    pub fn nyquist_velocity_raw(&self) -> u16 {
        self.header.nyquist_velocity.get()
    }

    /// Nyquist velocity, in m/s.
    pub fn nyquist_velocity_mps(&self) -> f32 {
        self.nyquist_velocity_raw() as f32 * 0.01
    }

    /// Nyquist velocity.
    #[cfg(feature = "uom")]
    pub fn nyquist_velocity(&self) -> uom::si::f64::Velocity {
        uom::si::f64::Velocity::new::<uom::si::velocity::meter_per_second>(
            self.nyquist_velocity_raw() as f64 * 0.01,
        )
    }

    /// Atmospheric attenuation factor in thousandths of a dB/km (raw scaled value). See
    /// [`atmospheric_attenuation_db_per_km()`](Self::atmospheric_attenuation_db_per_km) for the
    /// decoded value.
    pub fn atmospheric_attenuation_raw(&self) -> i16 {
        self.header.atmospheric_attenuation.get()
    }

    /// Atmospheric attenuation factor, in dB/km.
    pub fn atmospheric_attenuation_db_per_km(&self) -> f32 {
        self.atmospheric_attenuation_raw() as f32 * 0.001
    }

    /// System gain calibration constant in dB.
    pub fn calibration_constant_db(&self) -> f32 {
        self.header.calibration_constant.get()
    }

    /// Number of surveillance (reflectivity) gates.
    pub fn surveillance_gate_count(&self) -> u16 {
        self.header.surveillance_gate_count.get()
    }

    /// Number of Doppler (velocity/spectrum width) gates.
    pub fn doppler_gate_count(&self) -> u16 {
        self.header.doppler_gate_count.get()
    }

    /// Surveillance gate spacing in meters.
    pub fn surveillance_gate_interval_meters(&self) -> u16 {
        self.header.surveillance_gate_interval.get()
    }

    /// Doppler gate spacing in meters.
    pub fn doppler_gate_interval_meters(&self) -> u16 {
        self.header.doppler_gate_interval.get()
    }

    /// Range to first surveillance gate in meters.
    pub fn surveillance_first_gate_range_meters(&self) -> i16 {
        self.header.surveillance_first_gate_range.get()
    }

    /// Range to first Doppler gate in meters.
    pub fn doppler_first_gate_range_meters(&self) -> i16 {
        self.header.doppler_first_gate_range.get()
    }

    /// Doppler velocity resolution in m/s (0.5 or 1.0).
    pub fn doppler_velocity_resolution_mps(&self) -> f32 {
        match self.header.doppler_velocity_resolution.get() {
            4 => 1.0,
            _ => 0.5,
        }
    }

    /// Raw reflectivity gate data (1 byte per gate).
    ///
    /// Returns `None` if no reflectivity data is present in this radial.
    /// See [`Message`] documentation for gate value encoding.
    pub fn reflectivity_gates(&self) -> Option<&[u8]> {
        self.reflectivity_gates.as_deref()
    }

    /// Raw velocity gate data (1 byte per gate).
    ///
    /// Returns `None` if no velocity data is present in this radial.
    /// See [`Message`] documentation for gate value encoding.
    pub fn velocity_gates(&self) -> Option<&[u8]> {
        self.velocity_gates.as_deref()
    }

    /// Raw spectrum width gate data (1 byte per gate).
    ///
    /// Returns `None` if no spectrum width data is present in this radial.
    /// See [`Message`] documentation for gate value encoding.
    pub fn spectrum_width_gates(&self) -> Option<&[u8]> {
        self.spectrum_width_gates.as_deref()
    }

    /// Convert this message to a common model radial. This clones the underlying gate data;
    /// use [`into_radial`](Self::into_radial) to avoid the copy when the message is no longer
    /// needed.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Model`](crate::result::Error::Model) if the decoded values are invalid for
    /// a model radial.
    #[cfg(feature = "model")]
    pub fn radial(&self) -> Result<nexrad_model::data::Radial> {
        Self::build_radial(
            self,
            self.reflectivity_gates.as_deref().map(|g| g.to_vec()),
            self.velocity_gates.as_deref().map(|g| g.to_vec()),
            self.spectrum_width_gates.as_deref().map(|g| g.to_vec()),
        )
    }

    /// Convert this message into a common model radial, consuming the gate data without copying.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`radial`](Self::radial).
    #[cfg(feature = "model")]
    pub fn into_radial(self) -> Result<nexrad_model::data::Radial> {
        use crate::result::Error;
        use nexrad_model::data::{Product, Radial};

        // Extract header values before moving gate data out of self.
        let surveillance_gate_count = self.surveillance_gate_count();
        let surv_first_range = self.surveillance_first_gate_range_meters();
        let surv_interval = self.surveillance_gate_interval_meters();
        let doppler_gate_count = self.doppler_gate_count();
        let dopp_first_range = self.doppler_first_gate_range_meters();
        let dopp_interval = self.doppler_gate_interval_meters();
        let vel_resolution = self.doppler_velocity_resolution_mps();
        let radial_status: nexrad_model::data::RadialStatus = self.radial_status().into();
        let timestamp = self
            .date_time()
            .ok_or(Error::MessageMissingDate {
                message_type: crate::messages::MessageType::DigitalRadarDataLegacy,
            })?
            .timestamp_millis();
        let azimuth_number = self.azimuth_number();
        let azimuth_angle = self.azimuth_angle_degrees();
        let elevation_number = self.elevation_number();
        let elevation_angle = self.elevation_angle_degrees();
        let unambiguous_range_km = self.unambiguous_range_km();
        let nyquist_velocity_mps = self.nyquist_velocity_mps();

        let reflectivity = self
            .reflectivity_gates
            .map(|gates| {
                legacy_moment(
                    Product::Reflectivity,
                    surveillance_gate_count,
                    surv_first_range,
                    surv_interval,
                    2.0,
                    66.0,
                    gates.into_owned(),
                )
            })
            .transpose()?;

        let velocity = self
            .velocity_gates
            .map(|gates| {
                let scale = if vel_resolution > 0.9 { 1.0 } else { 2.0 };
                legacy_moment(
                    Product::Velocity,
                    doppler_gate_count,
                    dopp_first_range,
                    dopp_interval,
                    scale,
                    129.0,
                    gates.into_owned(),
                )
            })
            .transpose()?;

        let spectrum_width = self
            .spectrum_width_gates
            .map(|gates| {
                legacy_moment(
                    Product::SpectrumWidth,
                    doppler_gate_count,
                    dopp_first_range,
                    dopp_interval,
                    2.0,
                    129.0,
                    gates.into_owned(),
                )
            })
            .transpose()?;

        let mut builder = Radial::builder(
            azimuth_number,
            azimuth_angle,
            elevation_number,
            elevation_angle,
        )
        .collection_timestamp_millis(timestamp)
        .azimuth_spacing_degrees(1.0)
        .radial_status(radial_status);
        if nyquist_velocity_mps > 0.0 {
            builder = builder.nyquist_velocity_mps(nyquist_velocity_mps);
        }
        if unambiguous_range_km > 0.0 {
            builder = builder.unambiguous_range_km(unambiguous_range_km);
        }
        for moment in [reflectivity, velocity, spectrum_width]
            .into_iter()
            .flatten()
        {
            builder = builder.moment(moment);
        }

        Ok(builder.build()?)
    }

    #[cfg(feature = "model")]
    fn build_radial(
        msg: &Self,
        reflectivity_gates: Option<Vec<u8>>,
        velocity_gates: Option<Vec<u8>>,
        spectrum_width_gates: Option<Vec<u8>>,
    ) -> Result<nexrad_model::data::Radial> {
        use crate::result::Error;
        use nexrad_model::data::{Product, Radial};

        let reflectivity = reflectivity_gates
            .map(|gates| {
                legacy_moment(
                    Product::Reflectivity,
                    msg.surveillance_gate_count(),
                    msg.surveillance_first_gate_range_meters(),
                    msg.surveillance_gate_interval_meters(),
                    2.0,
                    66.0,
                    gates,
                )
            })
            .transpose()?;

        let velocity = velocity_gates
            .map(|gates| {
                let (scale, offset) = if msg.doppler_velocity_resolution_mps() > 0.9 {
                    (1.0, 129.0)
                } else {
                    (2.0, 129.0)
                };
                legacy_moment(
                    Product::Velocity,
                    msg.doppler_gate_count(),
                    msg.doppler_first_gate_range_meters(),
                    msg.doppler_gate_interval_meters(),
                    scale,
                    offset,
                    gates,
                )
            })
            .transpose()?;

        let spectrum_width = spectrum_width_gates
            .map(|gates| {
                legacy_moment(
                    Product::SpectrumWidth,
                    msg.doppler_gate_count(),
                    msg.doppler_first_gate_range_meters(),
                    msg.doppler_gate_interval_meters(),
                    2.0,
                    129.0,
                    gates,
                )
            })
            .transpose()?;

        let timestamp = msg
            .date_time()
            .ok_or(Error::MessageMissingDate {
                message_type: crate::messages::MessageType::DigitalRadarDataLegacy,
            })?
            .timestamp_millis();

        let mut builder = Radial::builder(
            msg.azimuth_number(),
            msg.azimuth_angle_degrees(),
            msg.elevation_number(),
            msg.elevation_angle_degrees(),
        )
        .collection_timestamp_millis(timestamp)
        .azimuth_spacing_degrees(1.0)
        .radial_status(msg.radial_status().into());
        if msg.nyquist_velocity_mps() > 0.0 {
            builder = builder.nyquist_velocity_mps(msg.nyquist_velocity_mps());
        }
        if msg.unambiguous_range_km() > 0.0 {
            builder = builder.unambiguous_range_km(msg.unambiguous_range_km());
        }
        for moment in [reflectivity, velocity, spectrum_width]
            .into_iter()
            .flatten()
        {
            builder = builder.moment(moment);
        }

        Ok(builder.build()?)
    }

    /// Convert this message to an owned version with `'static` lifetime.
    pub fn into_owned(self) -> Message<'static> {
        Message {
            header: Cow::Owned(self.header.into_owned()),
            reflectivity_gates: self.reflectivity_gates.map(|g| Cow::Owned(g.into_owned())),
            velocity_gates: self.velocity_gates.map(|g| Cow::Owned(g.into_owned())),
            spectrum_width_gates: self
                .spectrum_width_gates
                .map(|g| Cow::Owned(g.into_owned())),
        }
    }
}

/// Builds a model moment from a legacy radial's 8-bit gates.
///
/// The ICD defines the range to the first gate as a signed value, and type 1 radials can place
/// their first gates at a negative range (behind the radar). The model's first gate range is
/// unsigned, so leading gates centered at a negative range are dropped and the first gate range
/// is moved to the first remaining gate, which keeps every remaining gate at its decoded range.
#[cfg(feature = "model")]
fn legacy_moment(
    product: nexrad_model::data::Product,
    gate_count: u16,
    first_gate_range_meters: i16,
    gate_interval_meters: u16,
    scale: f32,
    offset: f32,
    mut gates: Vec<u8>,
) -> Result<nexrad_model::data::MomentData> {
    let (first_gate_range_meters, skipped) =
        first_gate_layout(first_gate_range_meters, gate_interval_meters)?;
    let gate_count = usize::from(gate_count).saturating_sub(skipped);
    gates.drain(..skipped.min(gates.len()));
    Ok(nexrad_model::data::MomentData::from_fixed_point(
        product,
        gate_count,
        first_gate_range_meters,
        gate_interval_meters,
        8,
        scale,
        offset,
        gates,
    )?)
}

/// The first non-negative gate's range in meters, and the number of leading gates before it.
///
/// # Errors
///
/// Returns [`Error::NegativeFirstGateRange`](crate::result::Error::NegativeFirstGateRange) for a
/// negative range with a zero gate interval, where no gate lies at a non-negative range.
#[cfg(feature = "model")]
fn first_gate_layout(range_meters: i16, interval_meters: u16) -> Result<(u16, usize)> {
    if let Ok(range) = u16::try_from(range_meters) {
        return Ok((range, 0));
    }
    if interval_meters == 0 {
        return Err(crate::result::Error::NegativeFirstGateRange { range_meters });
    }

    let behind = u32::from(range_meters.unsigned_abs());
    let interval = u32::from(interval_meters);
    let skipped = behind.div_ceil(interval);
    // Lands in 0..interval, so it fits in u16.
    let first = (skipped * interval - behind) as u16;
    Ok((first, skipped as usize))
}

impl Debug for Message<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DigitalRadarDataLegacy")
            .field("azimuth_angle_degrees", &self.azimuth_angle_degrees())
            .field("azimuth_number", &self.azimuth_number())
            .field("elevation_angle_degrees", &self.elevation_angle_degrees())
            .field("elevation_number", &self.elevation_number())
            .field("radial_status", &self.radial_status())
            .field("vcp_number", &self.vcp_number())
            .field("unambiguous_range_km", &self.unambiguous_range_km())
            .field("nyquist_velocity_mps", &self.nyquist_velocity_mps())
            .field(
                "atmospheric_attenuation_db_per_km",
                &self.atmospheric_attenuation_db_per_km(),
            )
            .field("calibration_constant_db", &self.calibration_constant_db())
            .field("surveillance_gate_count", &self.surveillance_gate_count())
            .field("doppler_gate_count", &self.doppler_gate_count())
            .field(
                "surveillance_gate_interval_meters",
                &self.surveillance_gate_interval_meters(),
            )
            .field(
                "doppler_gate_interval_meters",
                &self.doppler_gate_interval_meters(),
            )
            .field(
                "doppler_velocity_resolution_mps",
                &self.doppler_velocity_resolution_mps(),
            )
            .field("has_reflectivity", &self.reflectivity_gates.is_some())
            .field("has_velocity", &self.velocity_gates.is_some())
            .field("has_spectrum_width", &self.spectrum_width_gates.is_some())
            .finish()
    }
}

#[cfg(all(test, feature = "model"))]
mod tests {
    use super::*;
    use nexrad_model::data::Product;

    #[test]
    fn first_gate_layout_keeps_non_negative_ranges() {
        assert_eq!(first_gate_layout(0, 1000).unwrap(), (0, 0));
        assert_eq!(first_gate_layout(2125, 250).unwrap(), (2125, 0));
        assert_eq!(
            first_gate_layout(i16::MAX, 0).unwrap(),
            (i16::MAX as u16, 0)
        );
    }

    #[test]
    fn first_gate_layout_skips_gates_behind_the_radar() {
        // Legacy Doppler gates: centered at -375, -125, 125, ... meters.
        assert_eq!(first_gate_layout(-375, 250).unwrap(), (125, 2));
        assert_eq!(first_gate_layout(-500, 250).unwrap(), (0, 2));
        assert_eq!(first_gate_layout(-1, 1000).unwrap(), (999, 1));
        assert_eq!(first_gate_layout(i16::MIN, 1).unwrap(), (0, 32768));
    }

    #[test]
    fn first_gate_layout_rejects_negative_range_without_interval() {
        assert!(matches!(
            first_gate_layout(-375, 0),
            Err(crate::result::Error::NegativeFirstGateRange { range_meters: -375 })
        ));
    }

    #[test]
    fn legacy_moment_drops_gates_at_negative_range() {
        let moment = legacy_moment(
            Product::Velocity,
            4,
            -375,
            250,
            2.0,
            129.0,
            vec![10, 20, 30, 40],
        )
        .expect("builds moment");
        assert_eq!(moment.first_gate_range_meters(), 125);
        assert_eq!(moment.gate_interval_meters(), 250);
        assert_eq!(moment.gate_count(), 2);
        assert_eq!(moment.raw_values(), &[30, 40]);
    }

    #[test]
    fn legacy_moment_with_every_gate_behind_the_radar_is_empty() {
        let moment = legacy_moment(Product::Velocity, 2, -750, 250, 2.0, 129.0, vec![10, 20])
            .expect("builds moment");
        assert_eq!(moment.first_gate_range_meters(), 0);
        assert_eq!(moment.gate_count(), 0);
    }
}
