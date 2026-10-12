use crate::data::{ElevationCut, PulseWidth, VcpNumber};
use std::fmt::Display;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Volume Coverage Pattern (VCP) configuration describing how the radar scans a volume.
///
/// The VCP defines the scanning strategy including the number of elevation cuts, the settings
/// for each cut, and various special scanning modes like SAILS, MRLE, and MPDA.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VolumeCoveragePattern {
    pattern_number: VcpNumber,
    version: u8,
    doppler_velocity_resolution: f32,
    pulse_width: PulseWidth,

    // SAILS (Supplemental Adaptive Intra-volume Low-level Scan)
    sails_enabled: bool,
    sails_cuts: u8,

    // MRLE (Mid-volume Rescan of Low Elevation)
    mrle_enabled: bool,
    mrle_cuts: u8,

    // Other capabilities
    mpda_enabled: bool,
    base_tilt_enabled: bool,
    base_tilt_count: u8,

    // Sequencing
    sequence_active: bool,
    truncated: bool,

    // Per-elevation configuration
    elevation_cuts: Vec<ElevationCut>,
}

impl VolumeCoveragePattern {
    /// Start building a volume coverage pattern for the given pattern number.
    ///
    /// All other properties take the defaults documented on [`VolumeCoveragePatternBuilder`].
    pub fn builder(pattern_number: VcpNumber) -> VolumeCoveragePatternBuilder {
        VolumeCoveragePatternBuilder {
            vcp: VolumeCoveragePattern {
                pattern_number,
                version: 0,
                doppler_velocity_resolution: 0.5,
                pulse_width: PulseWidth::Short,
                sails_enabled: false,
                sails_cuts: 0,
                mrle_enabled: false,
                mrle_cuts: 0,
                mpda_enabled: false,
                base_tilt_enabled: false,
                base_tilt_count: 0,
                sequence_active: false,
                truncated: false,
                elevation_cuts: Vec::new(),
            },
        }
    }

    /// The volume coverage pattern identifying the scan strategy.
    pub fn pattern_number(&self) -> VcpNumber {
        self.pattern_number
    }

    /// The VCP version number.
    pub fn version(&self) -> u8 {
        self.version
    }

    /// Doppler velocity resolution in m/s (typically 0.5 or 1.0).
    pub fn doppler_velocity_resolution(&self) -> f32 {
        self.doppler_velocity_resolution
    }

    /// The pulse width configuration.
    pub fn pulse_width(&self) -> PulseWidth {
        self.pulse_width
    }

    /// Whether SAILS (Supplemental Adaptive Intra-volume Low-level Scan) is enabled.
    ///
    /// SAILS provides additional low-level scans during the volume scan to improve
    /// temporal resolution of the lowest elevations.
    pub fn sails_enabled(&self) -> bool {
        self.sails_enabled
    }

    /// The number of SAILS cuts in this VCP.
    pub fn sails_cuts(&self) -> u8 {
        self.sails_cuts
    }

    /// Whether MRLE (Mid-volume Rescan of Low Elevation) is enabled.
    ///
    /// MRLE provides additional rescans of low elevations during the middle of a volume scan.
    pub fn mrle_enabled(&self) -> bool {
        self.mrle_enabled
    }

    /// The number of MRLE cuts in this VCP.
    pub fn mrle_cuts(&self) -> u8 {
        self.mrle_cuts
    }

    /// Whether MPDA (Multiple PRF Dealiasing Algorithm) is enabled.
    pub fn mpda_enabled(&self) -> bool {
        self.mpda_enabled
    }

    /// Whether base tilt scanning is enabled.
    pub fn base_tilt_enabled(&self) -> bool {
        self.base_tilt_enabled
    }

    /// The number of base tilts in this VCP.
    pub fn base_tilt_count(&self) -> u8 {
        self.base_tilt_count
    }

    /// Whether the VCP sequence is currently active.
    pub fn sequence_active(&self) -> bool {
        self.sequence_active
    }

    /// Whether the VCP was truncated.
    pub fn truncated(&self) -> bool {
        self.truncated
    }

    /// The elevation cuts comprising this volume coverage pattern.
    pub fn elevation_cuts(&self) -> &[ElevationCut] {
        &self.elevation_cuts
    }

    /// The number of elevation cuts in this VCP.
    pub fn number_of_elevation_cuts(&self) -> usize {
        self.elevation_cuts.len()
    }
}

impl Display for VolumeCoveragePattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (v{}, {} cuts",
            self.pattern_number,
            self.version,
            self.elevation_cuts.len()
        )?;

        let mut features = Vec::new();
        if self.sails_enabled {
            features.push(format!("SAILS({})", self.sails_cuts));
        }
        if self.mrle_enabled {
            features.push(format!("MRLE({})", self.mrle_cuts));
        }
        if self.mpda_enabled {
            features.push("MPDA".to_string());
        }

        if !features.is_empty() {
            write!(f, ", {}", features.join(", "))?;
        }

        write!(f, ")")
    }
}

/// Builder for [`VolumeCoveragePattern`], created with [`VolumeCoveragePattern::builder`].
///
/// The pattern number is required. All other properties are optional and default to:
///
/// - `version`: `0`
/// - `doppler_velocity_resolution`: `0.5`
/// - `pulse_width`: `PulseWidth::Short`
/// - `sails_enabled`: `false`
/// - `sails_cuts`: `0`
/// - `mrle_enabled`: `false`
/// - `mrle_cuts`: `0`
/// - `mpda_enabled`: `false`
/// - `base_tilt_enabled`: `false`
/// - `base_tilt_count`: `0`
/// - `sequence_active`: `false`
/// - `truncated`: `false`
/// - `elevation_cuts`: empty
#[derive(Debug, Clone)]
#[must_use]
pub struct VolumeCoveragePatternBuilder {
    vcp: VolumeCoveragePattern,
}

impl VolumeCoveragePatternBuilder {
    /// Set the VCP version number. Defaults to `0`.
    pub fn version(mut self, version: u8) -> Self {
        self.vcp.version = version;
        self
    }

    /// Set the Doppler velocity resolution in m/s. Defaults to `0.5`.
    pub fn doppler_velocity_resolution(mut self, doppler_velocity_resolution: f32) -> Self {
        self.vcp.doppler_velocity_resolution = doppler_velocity_resolution;
        self
    }

    /// Set the pulse width configuration. Defaults to `PulseWidth::Short`.
    pub fn pulse_width(mut self, pulse_width: PulseWidth) -> Self {
        self.vcp.pulse_width = pulse_width;
        self
    }

    /// Set whether SAILS (Supplemental Adaptive Intra-volume Low-level Scan) is enabled. Defaults to `false`.
    pub fn sails_enabled(mut self, sails_enabled: bool) -> Self {
        self.vcp.sails_enabled = sails_enabled;
        self
    }

    /// Set the number of SAILS cuts in this VCP. Defaults to `0`.
    pub fn sails_cuts(mut self, sails_cuts: u8) -> Self {
        self.vcp.sails_cuts = sails_cuts;
        self
    }

    /// Set whether MRLE (Mid-volume Rescan of Low Elevation) is enabled. Defaults to `false`.
    pub fn mrle_enabled(mut self, mrle_enabled: bool) -> Self {
        self.vcp.mrle_enabled = mrle_enabled;
        self
    }

    /// Set the number of MRLE cuts in this VCP. Defaults to `0`.
    pub fn mrle_cuts(mut self, mrle_cuts: u8) -> Self {
        self.vcp.mrle_cuts = mrle_cuts;
        self
    }

    /// Set whether MPDA (Multiple PRF Dealiasing Algorithm) is enabled. Defaults to `false`.
    pub fn mpda_enabled(mut self, mpda_enabled: bool) -> Self {
        self.vcp.mpda_enabled = mpda_enabled;
        self
    }

    /// Set whether base tilt scanning is enabled. Defaults to `false`.
    pub fn base_tilt_enabled(mut self, base_tilt_enabled: bool) -> Self {
        self.vcp.base_tilt_enabled = base_tilt_enabled;
        self
    }

    /// Set the number of base tilts in this VCP. Defaults to `0`.
    pub fn base_tilt_count(mut self, base_tilt_count: u8) -> Self {
        self.vcp.base_tilt_count = base_tilt_count;
        self
    }

    /// Set whether the VCP sequence is currently active. Defaults to `false`.
    pub fn sequence_active(mut self, sequence_active: bool) -> Self {
        self.vcp.sequence_active = sequence_active;
        self
    }

    /// Set whether the VCP was truncated. Defaults to `false`.
    pub fn truncated(mut self, truncated: bool) -> Self {
        self.vcp.truncated = truncated;
        self
    }

    /// Set the elevation cuts comprising this pattern, replacing any previously added cuts.
    /// Defaults to empty.
    pub fn elevation_cuts(mut self, elevation_cuts: Vec<ElevationCut>) -> Self {
        self.vcp.elevation_cuts = elevation_cuts;
        self
    }

    /// Append a single elevation cut to this pattern. Elevation cuts default to empty.
    pub fn elevation_cut(mut self, elevation_cut: ElevationCut) -> Self {
        self.vcp.elevation_cuts.push(elevation_cut);
        self
    }

    /// Build the [`VolumeCoveragePattern`].
    #[must_use]
    pub fn build(self) -> VolumeCoveragePattern {
        self.vcp
    }
}
