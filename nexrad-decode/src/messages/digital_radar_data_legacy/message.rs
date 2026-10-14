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

        let num_surv = header.num_surveillance_gates.get() as usize;
        let num_dopp = header.num_doppler_gates.get() as usize;
        let ref_ptr = header.reflectivity_pointer.get() as usize;
        let vel_ptr = header.velocity_pointer.get() as usize;
        let sw_ptr = header.spectrum_width_pointer.get() as usize;

        // The header is 100 bytes. Gate data starts at byte 100 of the message body.
        // Pointers are byte offsets from the start of the message body.
        let header_size = size_of::<raw::Header>();

        let reflectivity_gates = if ref_ptr > 0 && num_surv > 0 && ref_ptr >= header_size {
            // Skip any gap between header end and reflectivity data start
            let gap = ref_ptr - header_size;
            if gap > 0 {
                reader.advance(gap);
            }
            let gates = reader.take_slice::<u8>(num_surv)?;
            Some(Cow::Borrowed(gates))
        } else {
            None
        };

        let velocity_gates = if vel_ptr > 0 && num_dopp > 0 {
            // Calculate current position and skip to velocity data
            let current_pos = if ref_ptr > 0 && num_surv > 0 {
                ref_ptr + num_surv
            } else {
                header_size
            };
            let gap = vel_ptr.saturating_sub(current_pos);
            if gap > 0 {
                reader.advance(gap);
            }
            let gates = reader.take_slice::<u8>(num_dopp)?;
            Some(Cow::Borrowed(gates))
        } else {
            None
        };

        let spectrum_width_gates = if sw_ptr > 0 && num_dopp > 0 {
            // Calculate current position and skip to spectrum width data
            let current_pos = if vel_ptr > 0 && num_dopp > 0 {
                vel_ptr + num_dopp
            } else if ref_ptr > 0 && num_surv > 0 {
                ref_ptr + num_surv
            } else {
                header_size
            };
            let gap = sw_ptr.saturating_sub(current_pos);
            if gap > 0 {
                reader.advance(gap);
            }
            let gates = reader.take_slice::<u8>(num_dopp)?;
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
        RadialStatus::from_raw(u8::try_from(self.radial_status_raw()).unwrap_or(u8::MAX))
    }

    /// Volume Coverage Pattern number (raw value). See [`vcp_number()`](Self::vcp_number) for the
    /// decoded value.
    pub fn vcp_number_raw(&self) -> u16 {
        self.header.vcp_number.get()
    }

    /// Volume Coverage Pattern number.
    pub fn vcp_number(&self) -> VcpNumber {
        VcpNumber::from(self.vcp_number_raw())
    }

    /// Unambiguous range in kilometers.
    pub fn unambiguous_range_km(&self) -> f32 {
        self.header.unambiguous_range.get() as f32 / 10.0
    }

    /// System gain calibration constant in dB.
    pub fn calibration_constant_db(&self) -> f32 {
        self.header.calibration_constant.get()
    }

    /// Number of surveillance (reflectivity) gates.
    pub fn num_surveillance_gates(&self) -> u16 {
        self.header.num_surveillance_gates.get()
    }

    /// Number of Doppler (velocity/spectrum width) gates.
    pub fn num_doppler_gates(&self) -> u16 {
        self.header.num_doppler_gates.get()
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
    #[cfg(feature = "model")]
    pub fn into_radial(self) -> Result<nexrad_model::data::Radial> {
        use crate::result::Error;
        use nexrad_model::data::{MomentData, Product, Radial};

        // Extract header values before moving gate data out of self.
        let num_surv_gates = self.num_surveillance_gates();
        let surv_first_range = self.surveillance_first_gate_range_meters().unsigned_abs();
        let surv_interval = self.surveillance_gate_interval_meters();
        let num_dopp_gates = self.num_doppler_gates();
        let dopp_first_range = self.doppler_first_gate_range_meters().unsigned_abs();
        let dopp_interval = self.doppler_gate_interval_meters();
        let vel_resolution = self.doppler_velocity_resolution_mps();
        let radial_status: nexrad_model::data::RadialStatus = self.radial_status().into();
        let timestamp = self
            .date_time()
            .ok_or(Error::MessageMissingDate)?
            .timestamp_millis();
        let azimuth_number = self.azimuth_number();
        let azimuth_angle = self.azimuth_angle_degrees();
        let elevation_number = self.elevation_number();
        let elevation_angle = self.elevation_angle_degrees();
        let unambiguous_range_km = self.unambiguous_range_km();

        let reflectivity = self
            .reflectivity_gates
            .map(|gates| {
                MomentData::from_fixed_point(
                    Product::Reflectivity,
                    usize::from(num_surv_gates),
                    surv_first_range,
                    surv_interval,
                    8,
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
                MomentData::from_fixed_point(
                    Product::Velocity,
                    usize::from(num_dopp_gates),
                    dopp_first_range,
                    dopp_interval,
                    8,
                    scale,
                    129.0,
                    gates.into_owned(),
                )
            })
            .transpose()?;

        let spectrum_width = self
            .spectrum_width_gates
            .map(|gates| {
                MomentData::from_fixed_point(
                    Product::SpectrumWidth,
                    usize::from(num_dopp_gates),
                    dopp_first_range,
                    dopp_interval,
                    8,
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
        use nexrad_model::data::{MomentData, Product, Radial};

        let reflectivity = reflectivity_gates
            .map(|gates| {
                MomentData::from_fixed_point(
                    Product::Reflectivity,
                    usize::from(msg.num_surveillance_gates()),
                    msg.surveillance_first_gate_range_meters().unsigned_abs(),
                    msg.surveillance_gate_interval_meters(),
                    8,
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
                MomentData::from_fixed_point(
                    Product::Velocity,
                    usize::from(msg.num_doppler_gates()),
                    msg.doppler_first_gate_range_meters().unsigned_abs(),
                    msg.doppler_gate_interval_meters(),
                    8,
                    scale,
                    offset,
                    gates,
                )
            })
            .transpose()?;

        let spectrum_width = spectrum_width_gates
            .map(|gates| {
                MomentData::from_fixed_point(
                    Product::SpectrumWidth,
                    usize::from(msg.num_doppler_gates()),
                    msg.doppler_first_gate_range_meters().unsigned_abs(),
                    msg.doppler_gate_interval_meters(),
                    8,
                    2.0,
                    129.0,
                    gates,
                )
            })
            .transpose()?;

        let timestamp = msg
            .date_time()
            .ok_or(Error::MessageMissingDate)?
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
            .field("calibration_constant_db", &self.calibration_constant_db())
            .field("num_surveillance_gates", &self.num_surveillance_gates())
            .field("num_doppler_gates", &self.num_doppler_gates())
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
