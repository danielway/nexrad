use crate::volume::{split_records, Header, Record};
use std::borrow::Cow;
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
///   [`compressed`](Self::compressed) is true, [`records`](Self::records), which borrows its
///   records from the file, returns
///   [`Error::CompressedFile`](crate::result::Error::CompressedFile).
///   [`decompress`](Self::decompress) inflates the file, and returns an uncompressed file
///   unchanged. `scan` and [`coverage_pattern_message`](Self::coverage_pattern_message) inflate
///   a gzip-wrapped file themselves.
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
    /// calling [`records`](Self::records) if the file may be gzip-compressed; `scan` and
    /// [`coverage_pattern_message`](Self::coverage_pattern_message) accept either form.
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    /// Whether this file's data is gzip-compressed.
    ///
    /// Some older NEXRAD archive files (pre-~2016) are gzip-wrapped. These must
    /// be decompressed before [`records`](Self::records) can be accessed.
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
        if !self.compressed() {
            return Ok(self);
        }

        Ok(File(inflate_gzip(&self.0)?))
    }

    /// This file with any gzip wrapping inflated: the file itself when it is not compressed,
    /// or a decompressed copy when it is.
    pub(crate) fn inflated(&self) -> crate::result::Result<Cow<'_, File>> {
        if !self.compressed() {
            return Ok(Cow::Borrowed(self));
        }

        Ok(Cow::Owned(File(inflate_gzip(&self.0)?)))
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
    /// Searches the file's LDM records in order, inflating a gzip-wrapped file and decompressing
    /// each compressed record, and returns the first volume coverage pattern message found as an
    /// owned message. For a real-time Start chunk, this is the message that describes the
    /// volume's elevation cuts.
    ///
    /// # Errors
    ///
    /// Returns [`Error::MissingCoveragePattern`](crate::result::Error::MissingCoveragePattern) if
    /// no record carries one, and propagates gzip inflation, record splitting, decompression, and
    /// decoding errors otherwise.
    pub fn coverage_pattern_message(
        &self,
    ) -> crate::result::Result<nexrad_decode::messages::volume_coverage_pattern::Message<'static>>
    {
        use crate::result::Error;
        use nexrad_decode::messages::MessageContents;

        let file = self.inflated()?;
        for mut record in file.records()? {
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
    /// A gzip-wrapped file is inflated first, and compressed records are decompressed. Both
    /// modern (message type 31) and legacy (message type 1) radials are included. The scan's
    /// coverage pattern and coverage pattern number are chosen as
    /// [`ScanBuilder::build`](nexrad_decode::model::ScanBuilder::build) describes, so a volume
    /// without a volume coverage pattern message still produces a scan, with
    /// [`Scan::coverage_pattern`](nexrad_model::data::Scan::coverage_pattern) `None`.
    ///
    /// # Errors
    ///
    /// Propagates gzip inflation, record splitting and decompression errors. Any message that
    /// cannot be decoded or converted fails the whole scan with
    /// [`Error::Decode`](crate::result::Error::Decode) carrying the first such error, as does a
    /// volume with no coverage pattern number in a volume coverage pattern message or any radial
    /// ([`MissingCoveragePatternNumber`](nexrad_decode::result::Error::MissingCoveragePatternNumber)).
    #[cfg(feature = "model")]
    pub fn scan(&self) -> crate::result::Result<nexrad_model::data::Scan> {
        use nexrad_decode::model::ScanBuilder;

        let file = self.inflated()?;
        let records = file.records()?;

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
        Ok(scan.build()?)
    }
}

/// Inflates gzip-compressed data.
fn inflate_gzip(data: &[u8]) -> crate::result::Result<Vec<u8>> {
    use flate2::read::GzDecoder;
    use std::io::Read;

    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
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
