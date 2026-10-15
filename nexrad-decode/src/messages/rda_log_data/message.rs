use crate::messages::rda_log_data::raw::Header;
use crate::result::Result;
use crate::segmented_slice_reader::SegmentedSliceReader;
use std::borrow::Cow;
use std::fmt::Debug;

/// An RDA log data message (type 33) containing log file data from the RDA.
///
/// This message's contents correspond to ICD 2620002AA section 3.2.4.33 Table XIVV.
/// The message starts with a fixed header describing the log file metadata (version, identifier,
/// compression info) followed by the log data payload which may be compressed.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Message<'a> {
    /// Decoded header information for this log data message.
    header: Cow<'a, Header>,

    /// The log data payload bytes (may be compressed according to compression_type).
    data: Cow<'a, [u8]>,
}

impl<'a> Message<'a> {
    /// Parse an RDA log data message from segmented input.
    pub(crate) fn parse(reader: &mut SegmentedSliceReader<'a, '_>) -> Result<Self> {
        let header = reader.take_ref::<Header>()?;

        let data_size = header.compressed_size.get() as usize;
        let data = reader.take_slice::<u8>(data_size)?;

        Ok(Message {
            header: Cow::Borrowed(header),
            data: Cow::Borrowed(data),
        })
    }

    /// Version number of this log message format (1-10000).
    pub fn version(&self) -> u32 {
        self.header.version.get()
    }

    /// The log file identifier (26 raw bytes). See [`identifier()`](Self::identifier) for the
    /// decoded value.
    pub fn identifier_raw(&self) -> &[u8; 26] {
        &self.header.identifier
    }

    /// The log file identifier with trailing null bytes and spaces trimmed, or `None` if the
    /// bytes are not valid UTF-8.
    pub fn identifier(&self) -> Option<&str> {
        let bytes = &self.header.identifier;
        let end = bytes
            .iter()
            .rposition(|&b| b != 0 && b != b' ')
            .map_or(0, |i| i + 1);
        std::str::from_utf8(&bytes[..end]).ok()
    }

    /// Version number of the data payload (1-10000).
    pub fn data_version(&self) -> u32 {
        self.header.data_version.get()
    }

    /// The compression type used for the log data: 0 = Uncompressed, 1 = GZIP,
    /// 2 = BZIP2, 3 = ZIP (raw code).
    pub fn compression_type_raw(&self) -> u32 {
        self.header.compression_type.get()
    }

    /// The size of the compressed log data in bytes.
    pub fn compressed_size_bytes(&self) -> u32 {
        self.header.compressed_size.get()
    }

    /// The size of the log data when decompressed in bytes.
    pub fn decompressed_size_bytes(&self) -> u32 {
        self.header.decompressed_size.get()
    }

    /// The log data payload bytes (may be compressed according to compression_type).
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Convert this message to an owned version with `'static` lifetime.
    pub fn into_owned(self) -> Message<'static> {
        Message {
            header: Cow::Owned(self.header.into_owned()),
            data: Cow::Owned(self.data.into_owned()),
        }
    }
}
