use crate::messages::VcpNumber;

/// Where the RDA system's active volume coverage pattern was specified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VcpSource {
    /// The pattern was specified locally (negative value on the wire).
    Local,
    /// The pattern was specified remotely (positive value on the wire).
    Remote,
}

/// The RDA system's active volume coverage pattern and where it was specified.
///
/// On the wire this is a signed value whose magnitude is the pattern number and whose sign
/// indicates whether it was specified locally (negative) or remotely (positive). Zero means no
/// pattern is set and is represented by the absence of a `VcpSelection`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VcpSelection {
    number: VcpNumber,
    source: VcpSource,
}

impl VcpSelection {
    /// Interprets a raw signed wire value, returning `None` when no pattern is set (zero).
    pub(crate) fn from_raw(value: i16) -> Option<Self> {
        let source = match value {
            0 => return None,
            v if v < 0 => VcpSource::Local,
            _ => VcpSource::Remote,
        };

        Some(Self {
            number: VcpNumber::from_number(value.unsigned_abs()),
            source,
        })
    }

    /// The volume coverage pattern number.
    pub fn number(&self) -> VcpNumber {
        self.number
    }

    /// Where the volume coverage pattern was specified.
    pub fn source(&self) -> VcpSource {
        self.source
    }
}

#[cfg(test)]
mod tests {
    use super::{VcpSelection, VcpSource};
    use crate::messages::VcpNumber;

    #[test]
    fn interprets_sign_as_source() {
        assert_eq!(VcpSelection::from_raw(0), None);

        let local = VcpSelection::from_raw(-212).unwrap();
        assert_eq!(local.number(), VcpNumber::Vcp212);
        assert_eq!(local.source(), VcpSource::Local);

        let remote = VcpSelection::from_raw(35).unwrap();
        assert_eq!(remote.number(), VcpNumber::Vcp35);
        assert_eq!(remote.source(), VcpSource::Remote);

        let min = VcpSelection::from_raw(i16::MIN).unwrap();
        assert_eq!(min.number(), VcpNumber::Unknown(32768));
    }
}
