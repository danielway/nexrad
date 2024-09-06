use crate::result::{Error, Result};
use crate::volume::{split_compressed_records, Header, Record};
use nexrad_decode::messages::Message;
use nexrad_model::data::Radial;

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
    #[cfg(feature = "decode")]
    pub fn scan(&self) -> Result<nexrad_model::data::Scan> {
        use nexrad_model::data::{Scan, Sweep};

        let mut coverage_pattern = None;
        let mut sweeps = Vec::new();

        let mut sweep_elevation_number = None;
        let mut sweep_radials = Vec::new();

        for mut record in self.records() {
            if record.compressed() {
                record = record.decompress()?;
            }

            let radials: Vec<Radial> = record
                .messages()?
                .into_iter()
                .filter_map(|message| match message.message {
                    Message::DigitalRadarData(radar_data_message) => {
                        if coverage_pattern.is_none() {
                            coverage_pattern = radar_data_message
                                .volume_data_block
                                .as_ref()
                                .map(|block| block.volume_coverage_pattern_number);
                        }

                        radar_data_message.into_radial().ok()
                    }
                    _ => None,
                })
                .collect();

            for radial in radials {
                match sweep_elevation_number {
                    Some(current_sweep_elevation_number) => {
                        if current_sweep_elevation_number != radial.elevation_number() {
                            sweeps.push(Sweep::new(current_sweep_elevation_number, sweep_radials));

                            sweep_elevation_number = Some(radial.elevation_number());
                            sweep_radials = Vec::new();
                        }
                    }
                    None => {
                        sweep_elevation_number = Some(radial.elevation_number());
                    }
                }

                sweep_radials.push(radial);
            }
        }

        if let Some(elevation_number) = sweep_elevation_number {
            sweeps.push(Sweep::new(elevation_number, sweep_radials));
        }

        Ok(Scan::new(
            coverage_pattern.ok_or(Error::MissingCoveragePattern)?,
            sweeps,
        ))
    }
}
