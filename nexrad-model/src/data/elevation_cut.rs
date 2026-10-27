use crate::data::{ChannelConfiguration, WaveformType};
use crate::result::{Error, Result};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Configuration for a single elevation cut in a volume coverage pattern.
///
/// Each elevation cut defines the radar settings used when scanning at a particular elevation
/// angle, including waveform type, PRF settings, and signal-to-noise thresholds.
///
/// Build one with [`ElevationCut::builder`]. Every floating-point property is finite, which
/// [`ElevationCutBuilder::build`] and deserialization check.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct ElevationCut {
    elevation_angle_degrees: f32,
    channel_configuration: ChannelConfiguration,
    waveform_type: WaveformType,
    azimuth_rate_degrees_per_second: f64,

    // Super resolution flags
    super_resolution_half_degree_azimuth: bool,
    super_resolution_quarter_km_reflectivity: bool,
    super_resolution_doppler_to_300km: bool,
    super_resolution_dual_pol_to_300km: bool,

    // PRF settings
    surveillance_prf_number: u8,
    surveillance_prf_pulse_count: u16,

    // Signal-to-noise ratio thresholds (dB)
    reflectivity_snr_threshold_db: f32,
    velocity_snr_threshold_db: f32,
    spectrum_width_snr_threshold_db: f32,
    differential_reflectivity_snr_threshold_db: f32,
    differential_phase_snr_threshold_db: f32,
    correlation_coefficient_snr_threshold_db: f32,

    // Special cut flags
    is_sails_cut: bool,
    sails_sequence_number: u8,
    is_mrle_cut: bool,
    mrle_sequence_number: u8,
    is_mpda_cut: bool,
    is_base_tilt_cut: bool,
}

impl ElevationCut {
    /// Start building an elevation cut from its required configuration.
    ///
    /// All other properties take the defaults documented on [`ElevationCutBuilder`].
    pub fn builder(
        elevation_angle_degrees: f32,
        waveform_type: WaveformType,
        channel_configuration: ChannelConfiguration,
    ) -> ElevationCutBuilder {
        ElevationCutBuilder {
            cut: ElevationCut {
                elevation_angle_degrees,
                channel_configuration,
                waveform_type,
                azimuth_rate_degrees_per_second: 0.0,
                super_resolution_half_degree_azimuth: false,
                super_resolution_quarter_km_reflectivity: false,
                super_resolution_doppler_to_300km: false,
                super_resolution_dual_pol_to_300km: false,
                surveillance_prf_number: 0,
                surveillance_prf_pulse_count: 0,
                reflectivity_snr_threshold_db: 0.0,
                velocity_snr_threshold_db: 0.0,
                spectrum_width_snr_threshold_db: 0.0,
                differential_reflectivity_snr_threshold_db: 0.0,
                differential_phase_snr_threshold_db: 0.0,
                correlation_coefficient_snr_threshold_db: 0.0,
                is_sails_cut: false,
                sails_sequence_number: 0,
                is_mrle_cut: false,
                mrle_sequence_number: 0,
                is_mpda_cut: false,
                is_base_tilt_cut: false,
            },
        }
    }

    /// The elevation angle for this cut in degrees.
    pub fn elevation_angle_degrees(&self) -> f32 {
        self.elevation_angle_degrees
    }

    /// The channel configuration (phase coding) for this cut.
    pub fn channel_configuration(&self) -> ChannelConfiguration {
        self.channel_configuration
    }

    /// The waveform type used for this cut.
    pub fn waveform_type(&self) -> WaveformType {
        self.waveform_type
    }

    /// The azimuth rotation rate in degrees per second.
    pub fn azimuth_rate_degrees_per_second(&self) -> f64 {
        self.azimuth_rate_degrees_per_second
    }

    /// Whether 0.5 degree azimuth super resolution is enabled.
    pub fn super_resolution_half_degree_azimuth(&self) -> bool {
        self.super_resolution_half_degree_azimuth
    }

    /// Whether 1/4 km reflectivity super resolution is enabled.
    pub fn super_resolution_quarter_km_reflectivity(&self) -> bool {
        self.super_resolution_quarter_km_reflectivity
    }

    /// Whether Doppler to 300 km super resolution is enabled.
    pub fn super_resolution_doppler_to_300km(&self) -> bool {
        self.super_resolution_doppler_to_300km
    }

    /// Whether dual polarization to 300 km super resolution is enabled.
    pub fn super_resolution_dual_pol_to_300km(&self) -> bool {
        self.super_resolution_dual_pol_to_300km
    }

    /// The pulse repetition frequency number for surveillance cuts.
    pub fn surveillance_prf_number(&self) -> u8 {
        self.surveillance_prf_number
    }

    /// The pulse count per radial for surveillance cuts.
    pub fn surveillance_prf_pulse_count(&self) -> u16 {
        self.surveillance_prf_pulse_count
    }

    /// Signal to noise ratio threshold for reflectivity in dB.
    pub fn reflectivity_snr_threshold_db(&self) -> f32 {
        self.reflectivity_snr_threshold_db
    }

    /// Signal to noise ratio threshold for velocity in dB.
    pub fn velocity_snr_threshold_db(&self) -> f32 {
        self.velocity_snr_threshold_db
    }

    /// Signal to noise ratio threshold for spectrum width in dB.
    pub fn spectrum_width_snr_threshold_db(&self) -> f32 {
        self.spectrum_width_snr_threshold_db
    }

    /// Signal to noise ratio threshold for differential reflectivity in dB.
    pub fn differential_reflectivity_snr_threshold_db(&self) -> f32 {
        self.differential_reflectivity_snr_threshold_db
    }

    /// Signal to noise ratio threshold for differential phase in dB.
    pub fn differential_phase_snr_threshold_db(&self) -> f32 {
        self.differential_phase_snr_threshold_db
    }

    /// Signal to noise ratio threshold for correlation coefficient in dB.
    pub fn correlation_coefficient_snr_threshold_db(&self) -> f32 {
        self.correlation_coefficient_snr_threshold_db
    }

    /// Whether this is a SAILS (Supplemental Adaptive Intra-volume Low-level Scan) cut.
    pub fn is_sails_cut(&self) -> bool {
        self.is_sails_cut
    }

    /// The SAILS sequence number for this cut.
    pub fn sails_sequence_number(&self) -> u8 {
        self.sails_sequence_number
    }

    /// Whether this is an MRLE (Mid-volume Rescan of Low Elevation) cut.
    pub fn is_mrle_cut(&self) -> bool {
        self.is_mrle_cut
    }

    /// The MRLE sequence number for this cut.
    pub fn mrle_sequence_number(&self) -> u8 {
        self.mrle_sequence_number
    }

    /// Whether this is an MPDA (Multiple PRF Dealiasing Algorithm) cut.
    pub fn is_mpda_cut(&self) -> bool {
        self.is_mpda_cut
    }

    /// Whether this is a base tilt cut.
    pub fn is_base_tilt_cut(&self) -> bool {
        self.is_base_tilt_cut
    }
}

/// Builder for [`ElevationCut`], created with [`ElevationCut::builder`].
///
/// The elevation angle, waveform type, and channel configuration are required. All other
/// properties are optional and default to:
///
/// - `azimuth_rate_degrees_per_second`: `0.0`
/// - `super_resolution_half_degree_azimuth`: `false`
/// - `super_resolution_quarter_km_reflectivity`: `false`
/// - `super_resolution_doppler_to_300km`: `false`
/// - `super_resolution_dual_pol_to_300km`: `false`
/// - `surveillance_prf_number`: `0`
/// - `surveillance_prf_pulse_count`: `0`
/// - `reflectivity_snr_threshold_db`: `0.0`
/// - `velocity_snr_threshold_db`: `0.0`
/// - `spectrum_width_snr_threshold_db`: `0.0`
/// - `differential_reflectivity_snr_threshold_db`: `0.0`
/// - `differential_phase_snr_threshold_db`: `0.0`
/// - `correlation_coefficient_snr_threshold_db`: `0.0`
/// - `is_sails_cut`: `false`
/// - `sails_sequence_number`: `0`
/// - `is_mrle_cut`: `false`
/// - `mrle_sequence_number`: `0`
/// - `is_mpda_cut`: `false`
/// - `is_base_tilt_cut`: `false`
#[derive(Debug, Clone)]
#[must_use]
pub struct ElevationCutBuilder {
    cut: ElevationCut,
}

impl ElevationCutBuilder {
    /// Set the azimuth rotation rate in degrees per second. Defaults to `0.0`.
    pub fn azimuth_rate_degrees_per_second(mut self, azimuth_rate_degrees_per_second: f64) -> Self {
        self.cut.azimuth_rate_degrees_per_second = azimuth_rate_degrees_per_second;
        self
    }

    /// Set whether 0.5 degree azimuth super resolution is enabled. Defaults to `false`.
    pub fn super_resolution_half_degree_azimuth(
        mut self,
        super_resolution_half_degree_azimuth: bool,
    ) -> Self {
        self.cut.super_resolution_half_degree_azimuth = super_resolution_half_degree_azimuth;
        self
    }

    /// Set whether 1/4 km reflectivity super resolution is enabled. Defaults to `false`.
    pub fn super_resolution_quarter_km_reflectivity(
        mut self,
        super_resolution_quarter_km_reflectivity: bool,
    ) -> Self {
        self.cut.super_resolution_quarter_km_reflectivity =
            super_resolution_quarter_km_reflectivity;
        self
    }

    /// Set whether Doppler to 300 km super resolution is enabled. Defaults to `false`.
    pub fn super_resolution_doppler_to_300km(
        mut self,
        super_resolution_doppler_to_300km: bool,
    ) -> Self {
        self.cut.super_resolution_doppler_to_300km = super_resolution_doppler_to_300km;
        self
    }

    /// Set whether dual polarization to 300 km super resolution is enabled. Defaults to `false`.
    pub fn super_resolution_dual_pol_to_300km(
        mut self,
        super_resolution_dual_pol_to_300km: bool,
    ) -> Self {
        self.cut.super_resolution_dual_pol_to_300km = super_resolution_dual_pol_to_300km;
        self
    }

    /// Set the pulse repetition frequency number for surveillance cuts. Defaults to `0`.
    pub fn surveillance_prf_number(mut self, surveillance_prf_number: u8) -> Self {
        self.cut.surveillance_prf_number = surveillance_prf_number;
        self
    }

    /// Set the pulse count per radial for surveillance cuts. Defaults to `0`.
    pub fn surveillance_prf_pulse_count(mut self, surveillance_prf_pulse_count: u16) -> Self {
        self.cut.surveillance_prf_pulse_count = surveillance_prf_pulse_count;
        self
    }

    /// Set the signal to noise ratio threshold for reflectivity in dB. Defaults to `0.0`.
    pub fn reflectivity_snr_threshold_db(mut self, reflectivity_snr_threshold_db: f32) -> Self {
        self.cut.reflectivity_snr_threshold_db = reflectivity_snr_threshold_db;
        self
    }

    /// Set the signal to noise ratio threshold for velocity in dB. Defaults to `0.0`.
    pub fn velocity_snr_threshold_db(mut self, velocity_snr_threshold_db: f32) -> Self {
        self.cut.velocity_snr_threshold_db = velocity_snr_threshold_db;
        self
    }

    /// Set the signal to noise ratio threshold for spectrum width in dB. Defaults to `0.0`.
    pub fn spectrum_width_snr_threshold_db(mut self, spectrum_width_snr_threshold_db: f32) -> Self {
        self.cut.spectrum_width_snr_threshold_db = spectrum_width_snr_threshold_db;
        self
    }

    /// Set the signal to noise ratio threshold for differential reflectivity in dB. Defaults to `0.0`.
    pub fn differential_reflectivity_snr_threshold_db(
        mut self,
        differential_reflectivity_snr_threshold_db: f32,
    ) -> Self {
        self.cut.differential_reflectivity_snr_threshold_db =
            differential_reflectivity_snr_threshold_db;
        self
    }

    /// Set the signal to noise ratio threshold for differential phase in dB. Defaults to `0.0`.
    pub fn differential_phase_snr_threshold_db(
        mut self,
        differential_phase_snr_threshold_db: f32,
    ) -> Self {
        self.cut.differential_phase_snr_threshold_db = differential_phase_snr_threshold_db;
        self
    }

    /// Set the signal to noise ratio threshold for correlation coefficient in dB. Defaults to `0.0`.
    pub fn correlation_coefficient_snr_threshold_db(
        mut self,
        correlation_coefficient_snr_threshold_db: f32,
    ) -> Self {
        self.cut.correlation_coefficient_snr_threshold_db =
            correlation_coefficient_snr_threshold_db;
        self
    }

    /// Set whether this is a SAILS (Supplemental Adaptive Intra-volume Low-level Scan) cut. Defaults to `false`.
    pub fn is_sails_cut(mut self, is_sails_cut: bool) -> Self {
        self.cut.is_sails_cut = is_sails_cut;
        self
    }

    /// Set the SAILS sequence number for this cut. Defaults to `0`.
    pub fn sails_sequence_number(mut self, sails_sequence_number: u8) -> Self {
        self.cut.sails_sequence_number = sails_sequence_number;
        self
    }

    /// Set whether this is an MRLE (Mid-volume Rescan of Low Elevation) cut. Defaults to `false`.
    pub fn is_mrle_cut(mut self, is_mrle_cut: bool) -> Self {
        self.cut.is_mrle_cut = is_mrle_cut;
        self
    }

    /// Set the MRLE sequence number for this cut. Defaults to `0`.
    pub fn mrle_sequence_number(mut self, mrle_sequence_number: u8) -> Self {
        self.cut.mrle_sequence_number = mrle_sequence_number;
        self
    }

    /// Set whether this is an MPDA (Multiple PRF Dealiasing Algorithm) cut. Defaults to `false`.
    pub fn is_mpda_cut(mut self, is_mpda_cut: bool) -> Self {
        self.cut.is_mpda_cut = is_mpda_cut;
        self
    }

    /// Set whether this is a base tilt cut. Defaults to `false`.
    pub fn is_base_tilt_cut(mut self, is_base_tilt_cut: bool) -> Self {
        self.cut.is_base_tilt_cut = is_base_tilt_cut;
        self
    }

    /// Build the [`ElevationCut`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidCoveragePattern`] if the elevation angle, the azimuth rate or a
    /// signal-to-noise threshold is not finite. No range is imposed on the elevation angle: the
    /// ICD's angle encoding spans -180 to 180 degrees and the model passes decoded values on.
    pub fn build(self) -> Result<ElevationCut> {
        self.cut.validate()?;
        Ok(self.cut)
    }
}

impl ElevationCut {
    /// Checks the invariants documented on [`ElevationCutBuilder::build`].
    fn validate(&self) -> Result<()> {
        if !self.elevation_angle_degrees.is_finite() {
            return Err(Error::InvalidCoveragePattern {
                parameter: "elevation_angle_degrees",
                reason: "must be finite",
            });
        }
        if !self.azimuth_rate_degrees_per_second.is_finite() {
            return Err(Error::InvalidCoveragePattern {
                parameter: "azimuth_rate_degrees_per_second",
                reason: "must be finite",
            });
        }
        for (parameter, value) in [
            (
                "reflectivity_snr_threshold_db",
                self.reflectivity_snr_threshold_db,
            ),
            ("velocity_snr_threshold_db", self.velocity_snr_threshold_db),
            (
                "spectrum_width_snr_threshold_db",
                self.spectrum_width_snr_threshold_db,
            ),
            (
                "differential_reflectivity_snr_threshold_db",
                self.differential_reflectivity_snr_threshold_db,
            ),
            (
                "differential_phase_snr_threshold_db",
                self.differential_phase_snr_threshold_db,
            ),
            (
                "correlation_coefficient_snr_threshold_db",
                self.correlation_coefficient_snr_threshold_db,
            ),
        ] {
            if !value.is_finite() {
                return Err(Error::InvalidCoveragePattern {
                    parameter,
                    reason: "must be finite",
                });
            }
        }
        Ok(())
    }
}

/// Deserializes the derived field-by-field form, then applies the checks of
/// [`ElevationCutBuilder::build`].
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for ElevationCut {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let cut = ElevationCutRepr::deserialize(deserializer)?;
        cut.validate().map_err(serde::de::Error::custom)?;
        Ok(cut)
    }
}

/// The serialized form of [`ElevationCut`], validated before it is returned.
#[cfg(feature = "serde")]
#[derive(Deserialize)]
#[serde(remote = "ElevationCut", rename = "ElevationCut")]
struct ElevationCutRepr {
    elevation_angle_degrees: f32,
    channel_configuration: ChannelConfiguration,
    waveform_type: WaveformType,
    azimuth_rate_degrees_per_second: f64,

    // Super resolution flags
    super_resolution_half_degree_azimuth: bool,
    super_resolution_quarter_km_reflectivity: bool,
    super_resolution_doppler_to_300km: bool,
    super_resolution_dual_pol_to_300km: bool,

    // PRF settings
    surveillance_prf_number: u8,
    surveillance_prf_pulse_count: u16,

    // Signal-to-noise ratio thresholds (dB)
    reflectivity_snr_threshold_db: f32,
    velocity_snr_threshold_db: f32,
    spectrum_width_snr_threshold_db: f32,
    differential_reflectivity_snr_threshold_db: f32,
    differential_phase_snr_threshold_db: f32,
    correlation_coefficient_snr_threshold_db: f32,

    // Special cut flags
    is_sails_cut: bool,
    sails_sequence_number: u8,
    is_mrle_cut: bool,
    mrle_sequence_number: u8,
    is_mpda_cut: bool,
    is_base_tilt_cut: bool,
}
