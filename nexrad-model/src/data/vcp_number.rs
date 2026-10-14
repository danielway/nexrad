use std::fmt::Display;

/// Volume Coverage Pattern (VCP) number identifying the radar's scanning strategy.
///
/// Each VCP defines the number of elevation cuts, scan speed, pulse characteristics, and
/// range-unfolding techniques. The radar operator selects a VCP based on meteorological
/// conditions.
///
/// Serializes as its raw `u16` pattern number for compatibility with archival formats.
///
/// # Sources
///
/// Definitions are drawn from official NOAA/NWS documentation:
/// - NOAA JetStream: Volume Coverage Patterns
/// - WSR-88D Radar Operations Center VCP Info Sheet
/// - RDA/RPG ICD (2620002 series)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum VcpNumber {
    /// VCP 11: Precipitation mode. 14 elevation angles in ~5 minutes. The legacy severe-weather
    /// pattern in use before SZ-2 range unfolding; superseded by VCP 211 and VCP 212.
    Vcp11,

    /// VCP 12: Precipitation mode. 14 elevation angles in ~4.5 minutes. Fastest scan;
    /// default for severe weather. Does not use SZ-2 phase coding.
    Vcp12,

    /// VCP 21: Precipitation mode. 9 elevation angles in ~6 minutes. The legacy pattern for
    /// general (non-severe) precipitation; superseded by VCP 215.
    Vcp21,

    /// VCP 31: Clear air mode, long pulse. 5 elevation angles in ~10 minutes. The longer
    /// transmitted pulse increases sensitivity, permitting detection of very weak returns.
    Vcp31,

    /// VCP 32: Clear air mode, short pulse. 5 elevation angles in ~10 minutes. Identical
    /// scan angles to VCP 31 but uses short pulse for higher unambiguous velocity.
    Vcp32,

    /// VCP 35: Clear air mode with SZ-2. 9 elevation angles in ~7 minutes. Default clear
    /// air VCP. Scans the same lower angles as VCP 12 and applies SZ-2 range unfolding.
    Vcp35,

    /// VCP 112: Precipitation mode with MPDA. ~15 elevation angles in ~5.5 minutes.
    /// Employs both SZ-2 and Multiple PRF Dealiasing Algorithm. Designed for tropical
    /// cyclones and large-scale systems with widespread high velocities.
    Vcp112,

    /// VCP 121: Precipitation mode with MPDA. The 9 elevation angles of VCP 21, with the
    /// Multiple PRF Dealiasing Algorithm scanning the Doppler cuts at several PRFs to recover
    /// velocities beyond a single Nyquist interval. Used for widespread high-velocity events
    /// such as tropical cyclones.
    Vcp121,

    /// VCP 211: Precipitation mode with SZ-2. The 14 elevation angles of VCP 11, with SZ-2
    /// phase coding on the lowest tilts for second-trip echo recovery.
    Vcp211,

    /// VCP 212: Precipitation mode with SZ-2. 14-15 elevation angles in ~4.5-5 minutes.
    /// Applies SZ-2 phase coding on the lowest tilts for second-trip echo recovery.
    /// Supports up to 3 SAILS or 4 MRLE cuts.
    Vcp212,

    /// VCP 215: General surveillance / standard precipitation with SZ-2. 15 elevation
    /// angles in ~6 minutes. Combines low-angle coverage of VCP 12/212 with upper-angle
    /// coverage of the former VCP 11. Default for widespread rain or snow without intense
    /// thunderstorms.
    Vcp215,

    /// VCP 221: Precipitation mode with SZ-2. The 9 elevation angles of VCP 21, with SZ-2
    /// phase coding on the lowest tilts for second-trip echo recovery.
    Vcp221,

    /// A VCP number without a named variant, for forward compatibility with new scan
    /// strategies. [`from_number`](Self::from_number) never produces this for a number that has
    /// a named variant.
    Unknown(u16),
}

impl VcpNumber {
    /// Creates a `VcpNumber` from a raw `u16` pattern number.
    pub fn from_number(number: u16) -> Self {
        match number {
            11 => Self::Vcp11,
            12 => Self::Vcp12,
            21 => Self::Vcp21,
            31 => Self::Vcp31,
            32 => Self::Vcp32,
            35 => Self::Vcp35,
            112 => Self::Vcp112,
            121 => Self::Vcp121,
            211 => Self::Vcp211,
            212 => Self::Vcp212,
            215 => Self::Vcp215,
            221 => Self::Vcp221,
            other => Self::Unknown(other),
        }
    }

    /// Returns the raw `u16` pattern number.
    pub fn number(&self) -> u16 {
        match self {
            Self::Vcp11 => 11,
            Self::Vcp12 => 12,
            Self::Vcp21 => 21,
            Self::Vcp31 => 31,
            Self::Vcp32 => 32,
            Self::Vcp35 => 35,
            Self::Vcp112 => 112,
            Self::Vcp121 => 121,
            Self::Vcp211 => 211,
            Self::Vcp212 => 212,
            Self::Vcp215 => 215,
            Self::Vcp221 => 221,
            Self::Unknown(n) => *n,
        }
    }

    /// A human-readable description of the scan strategy.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Vcp11 => "Precipitation, 14 elevations",
            Self::Vcp12 => "Precipitation, fast update",
            Self::Vcp21 => "Precipitation, 9 elevations",
            Self::Vcp31 => "Clear air, long pulse",
            Self::Vcp32 => "Clear air, short pulse",
            Self::Vcp35 => "Clear air, SZ-2",
            Self::Vcp112 => "Precipitation, MPDA + SZ-2",
            Self::Vcp121 => "Precipitation, MPDA",
            Self::Vcp211 => "Precipitation, SZ-2",
            Self::Vcp212 => "Precipitation, SZ-2",
            Self::Vcp215 => "General surveillance, SZ-2",
            Self::Vcp221 => "Precipitation, SZ-2",
            Self::Unknown(_) => "Unknown",
        }
    }
}

impl Display for VcpNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown(n) => write!(f, "VCP {n}"),
            known => write!(f, "VCP {} ({})", known.number(), known.description()),
        }
    }
}

impl From<u16> for VcpNumber {
    fn from(number: u16) -> Self {
        Self::from_number(number)
    }
}

impl From<VcpNumber> for u16 {
    fn from(vcp: VcpNumber) -> Self {
        vcp.number()
    }
}

// Serialize as the raw u16 pattern number for backward compatibility with archival formats
// and existing snapshot tests.
#[cfg(feature = "serde")]
impl serde::Serialize for VcpNumber {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u16(self.number())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for VcpNumber {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let number = u16::deserialize(deserializer)?;
        Ok(Self::from_number(number))
    }
}
