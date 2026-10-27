use crate::volume::{split_records, Header, Record};
use std::fmt::Debug;

/// Gzip magic bytes (RFC 1952).
const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];

/// A NEXRAD Archive II volume data file.
///
/// # Compression
///
/// Data is compressed at two levels, and each level has its own state rule:
///
/// - **File (gzip).** Older archive files (pre-~2016) are gzip-wrapped. While
///   [`compressed`](Self::compressed) is true, [`records`](Self::records),
///   `scan` and [`coverage_pattern_message`](Self::coverage_pattern_message)
///   return [`Error::CompressedFile`](crate::result::Error::CompressedFile).
///   [`decompress`](Self::decompress) inflates the file, and returns an uncompressed file
///   unchanged.
/// - **Record (bzip2).** Each LDM [`Record`] is normally bzip2-compressed. While
///   [`Record::compressed`] is true, [`Record::messages`], `Record::radials` and
///   `Record::iter_radials` return
///   [`Error::CompressedRecord`](crate::result::Error::CompressedRecord).
///   [`Record::decompress`] inflates the record, and returns an uncompressed record
///   unchanged. `scan` and
///   [`coverage_pattern_message`](Self::coverage_pattern_message) decompress records
///   themselves.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct File(Vec<u8>);

impl File {
    /// Creates a new Archive II volume file with the provided data.
    ///
    /// The data is stored as-is. Call [`decompress`](Self::decompress) before
    /// accessing records if the file may be gzip-compressed.
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    /// Whether this file's data is gzip-compressed.
    ///
    /// Some older NEXRAD archive files (pre-~2016) are gzip-wrapped. These must
    /// be decompressed before records can be accessed.
    pub fn compressed(&self) -> bool {
        self.0.len() >= 2 && self.0[..2] == GZIP_MAGIC
    }

    /// Decompresses a gzip-compressed volume file.
    ///
    /// Returns a new `File` containing the decompressed data. If the file is not
    /// gzip-compressed, returns it unchanged.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`](crate::result::Error::Io) if the gzip data cannot be decompressed.
    pub fn decompress(self) -> crate::result::Result<File> {
        use flate2::read::GzDecoder;
        use std::io::Read;

        if !self.compressed() {
            return Ok(self);
        }

        let mut decoder = GzDecoder::new(self.0.as_slice());
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;

        Ok(File(decompressed))
    }

    /// The file's raw data.
    ///
    /// If the file is gzip-compressed, this returns the compressed bytes.
    /// Call [`decompress`](Self::decompress) first to get the decompressed content.
    pub fn data(&self) -> &[u8] {
        &self.0
    }

    /// The file's decoded Archive II volume header.
    ///
    /// Returns `None` if the header cannot be parsed (e.g. if the file is still
    /// gzip-compressed).
    pub fn header(&self) -> Option<Header> {
        Header::parse(&self.0)
    }

    /// The file's LDM records.
    ///
    /// # Errors
    ///
    /// Returns an error if the file is gzip-compressed (call [`decompress`](Self::decompress)
    /// first), if the data is shorter than the volume header, or if the record data is truncated
    /// or malformed.
    pub fn records(&self) -> crate::result::Result<Vec<Record<'_>>> {
        use crate::result::Error;

        if self.compressed() {
            return Err(Error::CompressedFile);
        }

        let record_data = self.0.get(Header::SIZE..).ok_or(Error::TruncatedRecord {
            expected: Header::SIZE,
            actual: self.0.len(),
        })?;

        split_records(record_data)
    }

    /// Decodes this file's volume coverage pattern (message type 5).
    ///
    /// Searches the file's LDM records in order, decompressing each compressed record, and returns
    /// the first volume coverage pattern message found as an owned message. For a real-time Start
    /// chunk, this is the message that describes the volume's elevation cuts.
    ///
    /// # Errors
    ///
    /// Returns [`Error::MissingCoveragePattern`](crate::result::Error::MissingCoveragePattern) if
    /// no record carries one, and propagates record splitting, decompression, and decoding errors
    /// otherwise (including [`Error::CompressedFile`](crate::result::Error::CompressedFile) for a
    /// gzip-compressed file; call [`decompress`](Self::decompress) first).
    pub fn coverage_pattern_message(
        &self,
    ) -> crate::result::Result<nexrad_decode::messages::volume_coverage_pattern::Message<'static>>
    {
        use crate::result::Error;
        use nexrad_decode::messages::MessageContents;

        for mut record in self.records()? {
            if record.compressed() {
                record = record.decompress()?;
            }

            for message in record.messages()? {
                if let MessageContents::RdaVolumeCoveragePattern(vcp) = message.contents() {
                    return Ok(vcp.clone().into_owned());
                }
            }
        }

        Err(Error::MissingCoveragePattern)
    }

    /// Decodes this volume file into a common model scan containing sweeps and radials with moment
    /// data.
    ///
    /// Both modern (message type 31) and legacy (message type 1) radials are included.
    ///
    /// # Errors
    ///
    /// Returns [`Error::MissingCoveragePattern`](crate::result::Error::MissingCoveragePattern) if
    /// the volume has no volume coverage pattern message, and propagates record splitting and
    /// decompression errors. Any message that cannot be decoded or converted fails the whole scan
    /// with [`Error::Decode`](crate::result::Error::Decode) carrying the first such error.
    #[cfg(feature = "model")]
    pub fn scan(&self) -> crate::result::Result<nexrad_model::data::Scan> {
        use nexrad_decode::model::ScanBuilder;

        let records = self.records()?;

        let decode_record = |record: Record<'_>| -> crate::result::Result<ScanBuilder> {
            let mut builder = ScanBuilder::new();
            add_record_to_scan(&mut builder, &record)?;
            Ok(builder)
        };

        #[cfg(feature = "parallel")]
        let builders: Vec<ScanBuilder> = {
            use rayon::prelude::*;
            records
                .into_par_iter()
                .map(decode_record)
                .collect::<crate::result::Result<Vec<_>>>()?
        };

        #[cfg(not(feature = "parallel"))]
        let builders: Vec<ScanBuilder> = records
            .into_iter()
            .map(decode_record)
            .collect::<crate::result::Result<Vec<_>>>()?;

        let mut scan = ScanBuilder::new();
        for builder in builders {
            scan.merge(builder);
        }
        build_scan(scan)
    }
}

/// Adds the messages of an LDM record, decompressing it first if needed, to a scan builder.
#[cfg(feature = "model")]
pub(crate) fn add_record_to_scan(
    builder: &mut nexrad_decode::model::ScanBuilder,
    record: &Record<'_>,
) -> crate::result::Result<()> {
    if record.compressed() {
        builder.add_messages(record.decompress()?.messages()?)?;
    } else {
        builder.add_messages(record.messages()?)?;
    }
    Ok(())
}

/// Builds a scan, surfacing a missing coverage pattern as
/// [`Error::MissingCoveragePattern`](crate::result::Error::MissingCoveragePattern) rather than a
/// wrapped decode error.
#[cfg(feature = "model")]
pub(crate) fn build_scan(
    builder: nexrad_decode::model::ScanBuilder,
) -> crate::result::Result<nexrad_model::data::Scan> {
    use crate::result::Error;
    use nexrad_decode::result::Error as DecodeError;

    builder.build().map_err(|error| match error {
        DecodeError::MissingCoveragePattern => Error::MissingCoveragePattern,
        error => Error::Decode(error),
    })
}

impl Debug for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("File");
        debug.field("data.len()", &self.data().len());
        debug.field("header", &self.header());

        #[cfg(feature = "model")]
        debug.field(
            "records.len()",
            &self.records().map(|records| records.len()),
        );

        debug.finish()
    }
}
