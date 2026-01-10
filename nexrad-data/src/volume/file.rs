use crate::volume::{split_compressed_records, Header, Record};
use std::fmt::Debug;
use zerocopy::Ref;

/// A NEXRAD Archive II volume data file.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct File(Vec<u8>);

impl File {
    /// Creates a new Archive II volume file with the provided data.
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    /// The file's encoded and compressed data.
    pub fn data(&self) -> &[u8] {
        &self.0
    }

    /// The file's decoded Archive II volume header.
    pub fn header(&self) -> Option<&Header> {
        Ref::<_, Header>::from_prefix(self.0.as_slice())
            .ok()
            .map(|(header, _rest)| Ref::into_ref(header))
    }

    /// The file's LDM records.
    pub fn records(&self) -> Vec<Record<'_>> {
        split_compressed_records(&self.0[size_of::<Header>()..])
    }

    /// Decodes this volume file into a common model scan containing sweeps and radials with moment
    /// data.
    #[cfg(feature = "nexrad-model")]
    pub fn scan(&self) -> crate::result::Result<nexrad_model::data::Scan> {
        use crate::result::Error;
        use nexrad_decode::messages::MessageContents;
        use nexrad_model::data::{Scan, Sweep};

        let mut coverage_pattern_number = None;
        let mut radials = Vec::new();
        for mut record in self.records() {
            if record.compressed() {
                record = record.decompress()?;
            }

            let messages = record.messages()?;
            for message in messages {
                let contents = message.into_contents();
                if let MessageContents::DigitalRadarData(radar_data_message) = contents {
                    if coverage_pattern_number.is_none() {
                        if let Some(volume_block) = &radar_data_message.volume_data_block {
                            coverage_pattern_number =
                                Some(volume_block.volume_coverage_pattern_number.get());
                        }
                    }

                    radials.push(radar_data_message.into_radial()?);
                }
            }
        }

        Ok(Scan::new(
            coverage_pattern_number.ok_or(Error::MissingCoveragePattern)?,
            Sweep::from_radials(radials),
        ))
    }
}

impl Debug for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("File");
        debug.field("data.len()", &self.data().len());
        debug.field("header", &self.header());

        #[cfg(feature = "nexrad-model")]
        debug.field("records.len()", &self.records().len());

        debug.finish()
    }
}
