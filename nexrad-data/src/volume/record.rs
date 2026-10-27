use std::fmt::Debug;

#[derive(Clone, PartialEq, Eq, Hash)]
enum RecordData<'a> {
    Borrowed(&'a [u8]),
    Owned(Vec<u8>),
}

impl Debug for RecordData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordData::Borrowed(data) => write!(f, "RecordData::Borrowed({} bytes)", data.len()),
            RecordData::Owned(data) => write!(f, "RecordData::Owned({} bytes)", data.len()),
        }
    }
}

/// Represents a single LDM record with its data which may be compressed.
///
/// The Unidata Local Data Manager (LDM) is a data distribution system used by the NWS to distribute
/// NEXRAD archival radar data. A NEXRAD "Archive II" file starts with an
/// [crate::volume::Header] followed by a series of compressed LDM records, each
/// containing messages with radar data.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Record<'a>(RecordData<'a>);

impl<'a> Record<'a> {
    /// Creates a new LDM record with the provided data.
    pub fn new(data: Vec<u8>) -> Self {
        Record(RecordData::Owned(data))
    }

    /// Creates a new LDM record with the provided data slice.
    pub fn from_slice(data: &'a [u8]) -> Self {
        Record(RecordData::Borrowed(data))
    }

    /// The data contained in this LDM record.
    pub fn data(&self) -> &[u8] {
        match &self.0 {
            RecordData::Borrowed(data) => data,
            RecordData::Owned(data) => data,
        }
    }

    /// Whether this LDM record's data is compressed.
    pub fn compressed(&self) -> bool {
        self.data().len() >= 6 && self.data()[4..6].as_ref() == b"BZ"
    }

    /// Decompresses this LDM record's data.
    ///
    /// Returns a new record containing the decompressed data. If the record is not compressed,
    /// returns an owned copy of it unchanged, as
    /// [`File::decompress`](crate::volume::File::decompress) does for a file, so calling this on a
    /// record of unknown state is safe.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`](crate::result::Error::Io) if the bzip2 data cannot be decompressed.
    pub fn decompress(&self) -> crate::result::Result<Record<'static>> {
        use bzip2::read::BzDecoder;
        use std::io::Read;

        if !self.compressed() {
            return Ok(Record::new(self.data().to_vec()));
        }

        // Skip the four-byte record size prefix
        let data = self.data().split_at(4).1;

        let mut decompressed_data = Vec::new();
        BzDecoder::new(data).read_to_end(&mut decompressed_data)?;

        Ok(Record::new(decompressed_data))
    }

    /// Decodes the NEXRAD level II messages contained in this LDM record.
    ///
    /// Decoding is strict, as in [`nexrad_decode::decode_messages`]: either every message in the
    /// record decodes or the first decoding error is returned. To keep the messages before a
    /// failure, pass [`data`](Self::data) to [`nexrad_decode::iter_messages`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::CompressedRecord`](crate::result::Error::CompressedRecord) if the record is
    /// still compressed, or [`Error::Decode`](crate::result::Error::Decode) with the error for the
    /// first message that cannot be decoded.
    pub fn messages(&self) -> crate::result::Result<Vec<nexrad_decode::messages::Message<'_>>> {
        use crate::result::Error;
        use nexrad_decode::messages::decode_messages;

        if self.compressed() {
            return Err(Error::CompressedRecord);
        }

        Ok(decode_messages(self.data())?)
    }

    /// Decodes the radar radials contained in this LDM record.
    ///
    /// This extracts all digital radar data messages (both modern type 31 and legacy type 1)
    /// from the record and converts them into [`Radial`](nexrad_model::data::Radial) objects.
    /// Non-radial messages are ignored.
    ///
    /// The record must be decompressed before calling this method.
    ///
    /// Decoding is all-or-nothing: if any message fails to decode, or any radar data message fails
    /// to convert to a radial, the call returns that first error and no radials. A legacy volume
    /// is a single record, so one bad radial fails the whole volume here. To keep the radials that
    /// do convert, use [`iter_radials`](Self::iter_radials).
    ///
    /// # Errors
    ///
    /// Returns the errors of [`messages`](Self::messages), and
    /// [`Error::Decode`](crate::result::Error::Decode) if a radar data message cannot be converted
    /// to a radial.
    #[cfg(feature = "model")]
    pub fn radials(&self) -> crate::result::Result<Vec<nexrad_model::data::Radial>> {
        self.iter_radials().collect()
    }

    /// Iterates over the radar radials contained in this LDM record, yielding one result per
    /// radar data message.
    ///
    /// Like [`radials`](Self::radials), this converts digital radar data messages (types 31 and 1)
    /// and ignores other messages, but it reports failures per item, so a caller can skip a
    /// radial that does not convert and keep the rest:
    ///
    /// - A radar data message that decodes but cannot be converted to a radial yields
    ///   [`Error::Decode`](crate::result::Error::Decode), and iteration continues with the next
    ///   message.
    /// - A message that cannot be decoded at all yields
    ///   [`Error::Decode`](crate::result::Error::Decode) and ends the iteration, since it gives no
    ///   reliable offset for the next message (see [`nexrad_decode::iter_messages`]).
    /// - A record that is still compressed yields a single
    ///   [`Error::CompressedRecord`](crate::result::Error::CompressedRecord).
    ///
    /// Collecting the iterator into a `Result<Vec<_>>` is equivalent to [`radials`](Self::radials).
    ///
    /// ```no_run
    /// # fn example(record: &nexrad_data::volume::Record<'_>) {
    /// let radials: Vec<_> = record
    ///     .iter_radials()
    ///     .filter_map(|radial| radial.inspect_err(|error| eprintln!("{error}")).ok())
    ///     .collect();
    /// # }
    /// ```
    #[cfg(feature = "model")]
    pub fn iter_radials(&self) -> RadialIter<'_> {
        let compressed = self.compressed();
        RadialIter {
            messages: (!compressed).then(|| nexrad_decode::iter_messages(self.data())),
            compressed,
        }
    }
}

/// An iterator over the radials in an LDM record, created by [`Record::iter_radials`].
///
/// Yields one result per radar data message; see [`Record::iter_radials`] for when it continues
/// after an error and when it stops.
#[cfg(feature = "model")]
#[derive(Debug, Clone)]
pub struct RadialIter<'a> {
    messages: Option<nexrad_decode::messages::MessageIter<'a>>,
    compressed: bool,
}

#[cfg(feature = "model")]
impl Iterator for RadialIter<'_> {
    type Item = crate::result::Result<nexrad_model::data::Radial>;

    fn next(&mut self) -> Option<Self::Item> {
        use nexrad_decode::messages::MessageContents;

        if self.compressed {
            self.compressed = false;
            return Some(Err(crate::result::Error::CompressedRecord));
        }

        let messages = self.messages.as_mut()?;
        for message in messages.by_ref() {
            let message = match message {
                Ok(message) => message,
                Err(error) => return Some(Err(error.into())),
            };
            let radial = match message.into_contents() {
                MessageContents::DigitalRadarData(m) => m.into_radial(),
                MessageContents::DigitalRadarDataLegacy(m) => m.into_radial(),
                _ => continue,
            };
            return Some(radial.map_err(Into::into));
        }
        None
    }
}

#[cfg(feature = "model")]
impl std::iter::FusedIterator for RadialIter<'_> {}

impl Debug for Record<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("Record");
        debug.field("data.len()", &self.data().len());
        debug.field(
            "borrowed",
            match &self.0 {
                RecordData::Borrowed(_) => &true,
                RecordData::Owned(_) => &false,
            },
        );
        debug.field("compressed", &self.compressed());
        debug.field(
            "messages.len()",
            &self.messages().map(|messages| messages.len()),
        );

        debug.finish()
    }
}

/// Splits record data into individual records.
///
/// Supports two archive formats:
/// - **Modern (LDM)**: Data is a series of size-prefixed, bzip2-compressed records.
///   Each record has a 4-byte big-endian size prefix followed by the compressed data.
/// - **Legacy (CTM)**: Data is a series of uncompressed 2432-byte CTM frames. Each
///   frame's first 12 bytes are the `rpg_unknown` field of the `MessageHeader`, so
///   the frames can be passed directly to the message decoder without stripping.
///   This format is used by older archive files (pre-~2016) and files with tape
///   filename version 01-04.
///
/// The format is auto-detected by checking whether the first 4 bytes form a valid
/// (non-zero) record size.
pub(crate) fn split_records(data: &[u8]) -> crate::result::Result<Vec<Record<'_>>> {
    if data.len() < 4 {
        // Not enough data for either format detection or a valid record.
        // Delegate to split_ldm_records which returns Ok(empty) for truly empty
        // data, or TruncatedRecord for 1-3 byte inputs.
        return split_ldm_records(data);
    }

    // Detect legacy CTM format: first 4 bytes are all zeros (no valid LDM size prefix).
    // In CTM frames, the first 12 bytes are the rpg_unknown field (zeros), whereas
    // in LDM records the first 4 bytes are a non-zero record size.
    let first_four = [data[0], data[1], data[2], data[3]];
    if first_four == [0, 0, 0, 0] {
        return split_ctm_frames(data);
    }

    split_ldm_records(data)
}

/// Splits modern LDM (Local Data Manager) size-prefixed records.
fn split_ldm_records(data: &[u8]) -> crate::result::Result<Vec<Record<'_>>> {
    use crate::result::Error;

    let mut records = Vec::new();

    let mut position = 0;
    loop {
        if position >= data.len() {
            break;
        }

        // Check bounds for reading record size
        if position + 4 > data.len() {
            return Err(Error::TruncatedRecord {
                expected: position + 4,
                actual: data.len(),
            });
        }

        let mut record_size_bytes = [0; 4];
        record_size_bytes.copy_from_slice(&data[position..position + 4]);
        let record_size = i32::from_be_bytes(record_size_bytes).unsigned_abs() as usize;

        // Validate record size is non-zero to prevent infinite loops
        if record_size == 0 {
            return Err(Error::InvalidRecordSize {
                size: record_size,
                offset: position,
            });
        }

        // Check bounds for full record
        let record_end = position + record_size + 4;
        if record_end > data.len() {
            return Err(Error::TruncatedRecord {
                expected: record_end,
                actual: data.len(),
            });
        }

        records.push(Record::from_slice(&data[position..record_end]));
        position = record_end;
    }

    Ok(records)
}

/// Returns legacy pre-LDM archive data as a single uncompressed record.
///
/// Legacy archive files (pre-~2016, tape filename versions 01-06) store messages
/// in a mixed format:
///
/// - **Overhead messages** (Types 2, 3, 5, 13, 15, 18, etc.) use fixed 2432-byte
///   frame-aligned segments, identical to modern LDM decompressed records.
/// - **Type 31 radial data** is contiguously packed at exactly `12 + seg_size * 2`
///   bytes per message, with NO frame alignment between radials.
///
/// The `decode_messages` function handles both modes natively: fixed-segment
/// messages consume exactly 2432 bytes per segment, and variable-length Type 31
/// messages consume their declared size. Empty padding frames (all zeros) between
/// message groups are harmlessly decoded as `MessageContents::Other`.
///
/// The data is NOT trimmed to 2432-byte boundaries because the contiguously-packed
/// Type 31 radials may extend past the last frame boundary.
fn split_ctm_frames(data: &[u8]) -> crate::result::Result<Vec<Record<'_>>> {
    if data.is_empty() {
        return Ok(Vec::new());
    }

    Ok(vec![Record::from_slice(data)])
}
