/// Known NEXRAD RDA build numbers.
///
/// These represent discrete software builds of the NEXRAD radar system, ranging
/// from Build 12.0 (ICD 2620002K) through Build 24.0 (ICD 2620002AA).
///
/// Build numbers are extracted from the RDA Status Data message (Type 2).
/// Different sites upgrade on their own schedules, so the same calendar date may
/// produce different build numbers from different radar sites.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum RdaBuildNumber {
    /// Build 12.0
    Build12_0,
    /// Build 13.0
    Build13_0,
    /// Build 14.0
    Build14_0,
    /// Build 15.0
    Build15_0,
    /// Build 16.0
    Build16_0,
    /// Build 17.0
    Build17_0,
    /// Build 18.0
    Build18_0,
    /// Build 19.0
    Build19_0,
    /// Build 20.0
    Build20_0,
    /// Build 21.0
    Build21_0,
    /// Build 22.0
    Build22_0,
    /// Build 23.0
    Build23_0,
    /// Build 24.0
    Build24_0,
    /// Unknown build number with its raw float value.
    Unknown(f32),
}

impl RdaBuildNumber {
    /// Create an RdaBuildNumber from the raw build number field value.
    ///
    /// The build number is stored as a scaled integer with two encoding schemes:
    ///   - Builds 12.0–20.0: raw value ÷ 10 (e.g. 190 → 19.0, 200 → 20.0)
    ///   - Builds 21.0+: raw value ÷ 100 (e.g. 2100 → 21.0, 2400 → 24.0)
    ///
    /// The heuristic is: if raw_value / 100 > 2.0, divide by 100; otherwise by 10.
    pub fn from_raw(raw_value: u16) -> Self {
        let value = raw_value as f32;
        let build = if value / 100.0 > 2.0 {
            value / 100.0
        } else {
            value / 10.0
        };

        // Round to nearest integer for matching known builds
        let build_int = build.round() as u16;

        match build_int {
            12 => RdaBuildNumber::Build12_0,
            13 => RdaBuildNumber::Build13_0,
            14 => RdaBuildNumber::Build14_0,
            15 => RdaBuildNumber::Build15_0,
            16 => RdaBuildNumber::Build16_0,
            17 => RdaBuildNumber::Build17_0,
            18 => RdaBuildNumber::Build18_0,
            19 => RdaBuildNumber::Build19_0,
            20 => RdaBuildNumber::Build20_0,
            21 => RdaBuildNumber::Build21_0,
            22 => RdaBuildNumber::Build22_0,
            23 => RdaBuildNumber::Build23_0,
            24 => RdaBuildNumber::Build24_0,
            _ => RdaBuildNumber::Unknown(build),
        }
    }

    /// Returns true if this build number is known.
    pub fn is_known(&self) -> bool {
        !matches!(self, RdaBuildNumber::Unknown(_))
    }

    /// Returns the build number as a float.
    pub fn as_float(&self) -> f32 {
        match self {
            RdaBuildNumber::Build12_0 => 12.0,
            RdaBuildNumber::Build13_0 => 13.0,
            RdaBuildNumber::Build14_0 => 14.0,
            RdaBuildNumber::Build15_0 => 15.0,
            RdaBuildNumber::Build16_0 => 16.0,
            RdaBuildNumber::Build17_0 => 17.0,
            RdaBuildNumber::Build18_0 => 18.0,
            RdaBuildNumber::Build19_0 => 19.0,
            RdaBuildNumber::Build20_0 => 20.0,
            RdaBuildNumber::Build21_0 => 21.0,
            RdaBuildNumber::Build22_0 => 22.0,
            RdaBuildNumber::Build23_0 => 23.0,
            RdaBuildNumber::Build24_0 => 24.0,
            RdaBuildNumber::Unknown(v) => *v,
        }
    }

    /// Returns true if this build uses the legacy 40-byte VolumeDataBlock format.
    ///
    /// Build 20.0 (ICD 2620002U, July 2021) expanded the VOL block from 40 to 48
    /// bytes, adding `zdr_bias_estimate_weighted_mean` and spare fields.
    pub fn uses_legacy_volume_data_block(&self) -> bool {
        self.as_float() < 20.0
    }
}
