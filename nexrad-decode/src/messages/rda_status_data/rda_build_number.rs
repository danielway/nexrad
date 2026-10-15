use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

/// An RDA software build number, as reported by the RDA Status Data message (Type 2).
///
/// The wire value is a scaled integer: when the value divided by 100 is greater than 2, the
/// build is the value divided by 100 (e.g. 2400 is build 24.0); otherwise it is the value divided
/// by 10 (e.g. 190 is build 19.0). Values that encode the same build with different scales
/// compare equal. Builds of 12.0 and later carry dual-polarisation data.
///
/// Different sites upgrade on their own schedules, so the same calendar date may produce
/// different build numbers from different radar sites.
#[derive(Debug, Clone, Copy)]
pub struct RdaBuildNumber(u16);

impl RdaBuildNumber {
    /// Create a build number from the raw wire value.
    pub const fn from_raw(raw: u16) -> Self {
        Self(raw)
    }

    /// The raw wire value.
    pub fn raw(self) -> u16 {
        self.0
    }

    /// The build number in hundredths of a build, e.g. 2400 for build 24.0.
    fn hundredths(self) -> u32 {
        if self.0 > 200 {
            u32::from(self.0)
        } else {
            u32::from(self.0) * 10
        }
    }

    /// The major build number, e.g. 24 for build 24.0.
    pub fn major(self) -> u16 {
        (self.hundredths() / 100) as u16
    }

    /// The minor build number: the digits after the decimal point, e.g. 0 for build 24.0 and 3
    /// for build 11.3.
    pub fn minor(self) -> u16 {
        let fraction = (self.hundredths() % 100) as u16;
        if fraction.is_multiple_of(10) {
            fraction / 10
        } else {
            fraction
        }
    }

    /// Returns true if this build uses the legacy 40-byte VolumeDataBlock format.
    ///
    /// Build 20.0 (ICD 2620002U, July 2021) expanded the VOL block from 40 to 48
    /// bytes, adding `zdr_bias_estimate_weighted_mean` and spare fields.
    pub fn uses_legacy_volume_data_block(&self) -> bool {
        self.hundredths() < 2000
    }
}

impl PartialEq for RdaBuildNumber {
    fn eq(&self, other: &Self) -> bool {
        self.hundredths() == other.hundredths()
    }
}

impl Eq for RdaBuildNumber {}

impl PartialOrd for RdaBuildNumber {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RdaBuildNumber {
    fn cmp(&self, other: &Self) -> Ordering {
        self.hundredths().cmp(&other.hundredths())
    }
}

impl Hash for RdaBuildNumber {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hundredths().hash(state);
    }
}

impl Display for RdaBuildNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let fraction = self.hundredths() % 100;
        if fraction.is_multiple_of(10) {
            write!(f, "{}.{}", self.major(), fraction / 10)
        } else {
            write!(f, "{}.{:02}", self.major(), fraction)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RdaBuildNumber;

    #[test]
    fn decodes_both_scales() {
        for (raw, major, minor, display) in [
            (120, 12, 0, "12.0"),
            (113, 11, 3, "11.3"),
            (190, 19, 0, "19.0"),
            (200, 20, 0, "20.0"),
            (2100, 21, 0, "21.0"),
            (2400, 24, 0, "24.0"),
            (2405, 24, 5, "24.05"),
        ] {
            let build = RdaBuildNumber::from_raw(raw);
            assert_eq!(build.raw(), raw);
            assert_eq!((build.major(), build.minor()), (major, minor), "raw {raw}");
            assert_eq!(build.to_string(), display);
        }
    }

    #[test]
    fn compares_by_build_across_scales() {
        assert_eq!(
            RdaBuildNumber::from_raw(190),
            RdaBuildNumber::from_raw(1900)
        );
        assert!(RdaBuildNumber::from_raw(200) > RdaBuildNumber::from_raw(1900));
        assert!(RdaBuildNumber::from_raw(2400) > RdaBuildNumber::from_raw(200));
    }

    #[test]
    fn legacy_volume_data_block_boundary() {
        assert!(RdaBuildNumber::from_raw(190).uses_legacy_volume_data_block());
        assert!(!RdaBuildNumber::from_raw(200).uses_legacy_volume_data_block());
        assert!(!RdaBuildNumber::from_raw(2100).uses_legacy_volume_data_block());
    }
}
