use crate::result::Result;
use crate::volume::{split_compressed_records, Header, Record};

/// A NEXRAD Archive II volume data file.
pub struct File(Vec<u8>);

impl File {
    /// Creates a new Archive II volume file with the provided data.
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    /// The file's encoded and compressed data.
    pub fn data(&self) -> &Vec<u8> {
        &self.0
    }

    /// The file's decoded Archive II volume header.
    pub fn header(&self) -> Result<Header> {
        Header::deserialize(&mut self.0.as_slice())
    }

    /// The file's LDM records.
    pub fn records(&self) -> Vec<Record> {
        split_compressed_records(&self.0[size_of::<Header>()..])
    }

    /// Decodes this volume file into a common model scan containing sweeps and radials with moment
    /// data.
    #[cfg(all(feature = "decode", feature = "bzip2"))]
    pub fn scan(&self) -> Result<nexrad_model::data::Scan> {
        use crate::result::Error;
        use nexrad_decode::messages::Message;
        use nexrad_model::data::{Scan, Sweep};

        let mut coverage_pattern_number = None;
        let mut radials = Vec::new();
        for mut record in self.records() {
            if record.compressed() {
                record = record.decompress()?;
            }

            let messages = record.messages()?;
            for message in messages {
                if let Message::DigitalRadarData(radar_data_message) = message.message {
                    if coverage_pattern_number.is_none() {
                        if let Some(volume_block) = &radar_data_message.volume_data_block {
                            coverage_pattern_number =
                                Some(volume_block.volume_coverage_pattern_number);
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
