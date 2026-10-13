use crate::volume::util::get_datetime;
use chrono::{DateTime, Duration, Utc};
use std::fmt::{self, Debug};
use zerocopy::FromBytes;

mod raw {
    use zerocopy::{big_endian, FromBytes, Immutable, KnownLayout};

    /// The Archive II volume header as it appears on the wire.
    #[repr(C)]
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, FromBytes, Immutable, KnownLayout)]
    pub(super) struct Header {
        /// The tape's filename which indicates the version of the data. Name is in the format
        /// `AR2V0 0xx.` where `xx` indicates the version of the data.
        ///
        /// Versions:
        ///   02 = Super Resolution disabled at the RDA (pre RDA Build 12.0)
        ///   03 = Super Resolution (pre RDA Build 12.0)
        ///   04 = Recombined Super Resolution
        ///   05 = Super Resolution disabled at the RDA (RDA Build 12.0 and later)
        ///   06 = Super Resolution (RDA Build 12.0 and later)
        ///   07 = Recombined Super Resolution (RDA Build 12.0 and later)
        /// NOTE: Dual-pol data introduced in RDA Build 12.0
        pub(super) tape_filename: [u8; 9],

        /// Sequential number assigned to each volume of radar data in the queue, rolling over to
        /// 001 after 999.
        pub(super) extension_number: [u8; 3],

        /// This volume's date represented as a count of days since 1 January 1970 00:00 GMT. It
        /// is also referred-to as a "modified Julian date" where it is the Julian date -
        /// 2440586.5.
        pub(super) date: big_endian::U32,

        /// Milliseconds past midnight, GMT.
        pub(super) time: big_endian::U32,

        /// The ICAO identifier of the radar site.
        pub(super) icao_of_radar: [u8; 4],
    }
}

const _: () = assert!(size_of::<raw::Header>() == Header::SIZE);

/// Header for an Archive II volume file containing metadata about the radar data. This header is
/// located at the beginning of the file.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Header {
    raw: raw::Header,
}

impl Header {
    /// The size of an Archive II volume header, in bytes. The file's LDM records start this many
    /// bytes into the volume file.
    pub const SIZE: usize = 24;

    /// Parse a volume header from the first [`SIZE`](Self::SIZE) bytes of `bytes`, copying it out
    /// of the input. Any bytes after the header are ignored.
    ///
    /// Returns `None` if `bytes` is shorter than [`SIZE`](Self::SIZE).
    pub(crate) fn parse(bytes: &[u8]) -> Option<Header> {
        let (raw, _rest) = raw::Header::read_from_prefix(bytes).ok()?;
        Some(Self { raw })
    }

    /// The tape's filename which indicates the version of the data. Name is in the format
    /// `AR2V0 0xx.` where `xx` indicates the version of the data.
    ///
    /// Versions:
    ///   02 = Super Resolution disabled at the RDA (pre RDA Build 12.0)
    ///   03 = Super Resolution (pre RDA Build 12.0)
    ///   04 = Recombined Super Resolution
    ///   05 = Super Resolution disabled at the RDA (RDA Build 12.0 and later)
    ///   06 = Super Resolution (RDA Build 12.0 and later)
    ///   07 = Recombined Super Resolution (RDA Build 12.0 and later)
    /// NOTE: Dual-pol data introduced in RDA Build 12.0
    pub fn tape_filename(&self) -> Option<&str> {
        std::str::from_utf8(&self.raw.tape_filename).ok()
    }

    /// Sequential number assigned to each volume of radar data in the queue, rolling over to 001
    /// after 999.
    pub fn extension_number(&self) -> Option<&str> {
        std::str::from_utf8(&self.raw.extension_number).ok()
    }

    /// Returns the date and time of the volume.
    pub fn date_time(&self) -> Option<DateTime<Utc>> {
        get_datetime(
            self.raw.date.get() as u16,
            Duration::milliseconds(self.raw.time.get() as i64),
        )
    }

    /// The ICAO identifier of the radar site.
    pub fn icao_of_radar(&self) -> Option<&str> {
        std::str::from_utf8(&self.raw.icao_of_radar).ok()
    }
}

impl Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Header")
            .field("tape_filename", &self.tape_filename())
            .field("extension_number", &self.extension_number())
            .field("date_time", &self.date_time())
            .field("icao_of_radar", &self.icao_of_radar())
            .finish()
    }
}
