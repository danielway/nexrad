use crate::volume::{split_compressed_records, Header, Record};
use std::fmt::Debug;
use zerocopy::Ref;

#[cfg(all(
    feature = "nexrad-model",
    any(feature = "parallel-decompress", feature = "parallel-decode")
))]
use rayon::prelude::*;

#[cfg(feature = "nexrad-model")]
use nexrad_decode::messages::volume_coverage_pattern as vcp;

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
    ///
    /// Returns an error if the record data is truncated or malformed.
    pub fn records(&self) -> crate::result::Result<Vec<Record<'_>>> {
        split_compressed_records(&self.0[size_of::<Header>()..])
    }

    /// Decodes this volume file into a common model scan containing sweeps and radials with moment
    /// data.
    #[cfg(feature = "nexrad-model")]
    pub fn scan(&self) -> crate::result::Result<nexrad_model::data::Scan> {
        #[cfg(any(feature = "parallel-decompress", feature = "parallel-decode"))]
        {
            return scan_parallel(self);
        }
        #[cfg(not(any(feature = "parallel-decompress", feature = "parallel-decode")))]
        {
            return scan_serial(self);
        }
    }
}

#[cfg(feature = "nexrad-model")]
#[cfg_attr(
    any(feature = "parallel-decompress", feature = "parallel-decode"),
    allow(dead_code)
)]
fn scan_serial(file: &File) -> crate::result::Result<nexrad_model::data::Scan> {
    use crate::result::Error;
    use nexrad_model::data::{Scan, Sweep};

    let records = decompress_records_serial(file.records()?)?;
    let mut coverage_pattern_message = None;
    let mut radials = Vec::new();

    for record in records {
        let messages = record.messages()?;
        let (record_vcp, mut record_radials) = decode_record_messages(messages)?;
        if coverage_pattern_message.is_none() {
            coverage_pattern_message = record_vcp;
        }
        radials.append(&mut record_radials);
    }

    let vcp_msg = coverage_pattern_message.ok_or(Error::MissingCoveragePattern)?;
    let coverage_pattern = coverage_pattern_from_message(&vcp_msg);

    Ok(Scan::new(coverage_pattern, Sweep::from_radials(radials)))
}

#[cfg(all(
    feature = "nexrad-model",
    any(feature = "parallel-decompress", feature = "parallel-decode")
))]
fn scan_parallel(file: &File) -> crate::result::Result<nexrad_model::data::Scan> {
    use crate::result::Error;
    use nexrad_model::data::{Scan, Sweep};

    let records = file.records()?;
    let records = {
        #[cfg(feature = "parallel-decompress")]
        {
            decompress_records_parallel(records)?
        }
        #[cfg(not(feature = "parallel-decompress"))]
        {
            decompress_records_serial(records)?
        }
    };

    let mut coverage_pattern_message = None;
    let mut radials = Vec::new();

    #[cfg(feature = "parallel-decode")]
    {
        let decoded_records = decode_records_parallel(&records)?;
        for decoded in decoded_records {
            let DecodedRecord {
                coverage_pattern_message: record_vcp,
                radials: record_radials,
                ..
            } = decoded;
            if coverage_pattern_message.is_none() {
                coverage_pattern_message = record_vcp;
            }
            radials.extend(record_radials);
        }
    }

    #[cfg(not(feature = "parallel-decode"))]
    {
        for record in records {
            let messages = record.messages()?;
            let (record_vcp, mut record_radials) = decode_record_messages(messages)?;
            if coverage_pattern_message.is_none() {
                coverage_pattern_message = record_vcp;
            }
            radials.append(&mut record_radials);
        }
    }

    let vcp_msg = coverage_pattern_message.ok_or(Error::MissingCoveragePattern)?;
    let coverage_pattern = coverage_pattern_from_message(&vcp_msg);

    Ok(Scan::new(coverage_pattern, Sweep::from_radials(radials)))
}

#[cfg(feature = "nexrad-model")]
fn decode_record_messages(
    messages: Vec<nexrad_decode::messages::Message<'_>>,
) -> crate::result::Result<(
    Option<vcp::Message<'static>>,
    Vec<nexrad_model::data::Radial>,
)> {
    use nexrad_decode::messages::MessageContents;

    let mut coverage_pattern_message: Option<vcp::Message<'static>> = None;
    let mut radials = Vec::new();

    for message in messages {
        match message.into_contents() {
            MessageContents::DigitalRadarData(radar_data_message) => {
                radials.push(radar_data_message.into_radial()?);
            }
            MessageContents::VolumeCoveragePattern(vcp_message) => {
                if coverage_pattern_message.is_none() {
                    coverage_pattern_message = Some(vcp_message.into_owned());
                }
            }
            _ => {}
        }
    }

    Ok((coverage_pattern_message, radials))
}

#[cfg(feature = "nexrad-model")]
fn coverage_pattern_from_message(
    vcp_msg: &vcp::Message<'_>,
) -> nexrad_model::data::VolumeCoveragePattern {
    use nexrad_model::data::{
        ChannelConfiguration, ElevationCut, PulseWidth, VolumeCoveragePattern, WaveformType,
    };

    let header = vcp_msg.header();

    let pulse_width = match header.pulse_width() {
        vcp::PulseWidth::Short => PulseWidth::Short,
        vcp::PulseWidth::Long => PulseWidth::Long,
        vcp::PulseWidth::Unknown => PulseWidth::Unknown,
    };

    let elevation_cuts: Vec<ElevationCut> = vcp_msg
        .elevations()
        .iter()
        .map(|elev| {
            let channel_config = match elev.channel_configuration() {
                vcp::ChannelConfiguration::ConstantPhase => ChannelConfiguration::ConstantPhase,
                vcp::ChannelConfiguration::RandomPhase => ChannelConfiguration::RandomPhase,
                vcp::ChannelConfiguration::SZ2Phase => ChannelConfiguration::SZ2Phase,
                vcp::ChannelConfiguration::UnknownPhase => ChannelConfiguration::Unknown,
            };

            let waveform = match elev.waveform_type() {
                vcp::WaveformType::CS => WaveformType::CS,
                vcp::WaveformType::CDW => WaveformType::CDW,
                vcp::WaveformType::CDWO => WaveformType::CDWO,
                vcp::WaveformType::B => WaveformType::B,
                vcp::WaveformType::SPP => WaveformType::SPP,
                vcp::WaveformType::Unknown => WaveformType::Unknown,
            };

            ElevationCut::new(
                elev.elevation_angle(),
                channel_config,
                waveform,
                elev.azimuth_rate(),
                elev.super_resolution_half_degree_azimuth(),
                elev.super_resolution_quarter_km_reflectivity(),
                elev.super_resolution_doppler_to_300km(),
                elev.super_resolution_dual_pol_to_300km(),
                elev.surveillance_prf_number(),
                elev.surveillance_prf_pulse_count_radial(),
                elev.reflectivity_threshold(),
                elev.velocity_threshold(),
                elev.spectrum_width_threshold(),
                elev.differential_reflectivity_threshold(),
                elev.differential_phase_threshold(),
                elev.correlation_coefficient_threshold(),
                elev.is_sails_cut(),
                elev.sails_sequence_number(),
                elev.is_mrle_cut(),
                elev.mrle_sequence_number(),
                elev.is_mpda_cut(),
                elev.is_base_tilt_cut(),
            )
        })
        .collect();

    VolumeCoveragePattern::new(
        header.pattern_number(),
        header.version(),
        header.doppler_velocity_resolution(),
        pulse_width,
        header.is_sails_vcp(),
        header.number_of_sails_cuts(),
        header.is_mrle_vcp(),
        header.number_of_mrle_cuts(),
        header.is_mpda_vcp(),
        header.is_base_tilt_vcp(),
        header.number_of_base_tilts(),
        header.vcp_sequencing_sequence_active(),
        header.vcp_sequencing_truncated(),
        elevation_cuts,
    )
}

#[cfg(feature = "nexrad-model")]
#[cfg_attr(
    any(feature = "parallel-decompress", feature = "parallel-decode"),
    allow(dead_code)
)]
fn decompress_records_serial<'a>(
    records: Vec<Record<'a>>,
) -> crate::result::Result<Vec<Record<'a>>> {
    let mut output = Vec::with_capacity(records.len());
    for record in records {
        if record.compressed() {
            output.push(record.decompress()?);
        } else {
            output.push(record);
        }
    }

    Ok(output)
}

#[cfg(all(feature = "nexrad-model", feature = "parallel-decompress"))]
fn decompress_records_parallel<'a>(
    records: Vec<Record<'a>>,
) -> crate::result::Result<Vec<Record<'a>>> {
    let results: Vec<_> = records
        .par_iter()
        .map(|record| {
            if record.compressed() {
                record.decompress()
            } else {
                Ok(record.clone())
            }
        })
        .collect();

    let mut output = Vec::with_capacity(results.len());
    for result in results {
        output.push(result?);
    }

    Ok(output)
}

#[cfg(all(feature = "nexrad-model", feature = "parallel-decode"))]
struct DecodedRecord {
    index: usize,
    coverage_pattern_message: Option<vcp::Message<'static>>,
    radials: Vec<nexrad_model::data::Radial>,
}

#[cfg(all(feature = "nexrad-model", feature = "parallel-decode"))]
fn decode_records_parallel<'a>(
    records: &[Record<'a>],
) -> crate::result::Result<Vec<DecodedRecord>> {
    let results: Vec<_> = records
        .par_iter()
        .enumerate()
        .map(|(index, record)| -> crate::result::Result<DecodedRecord> {
            let messages = record.messages()?;
            let (coverage_pattern_message, radials) = decode_record_messages(messages)?;
            Ok(DecodedRecord {
                index,
                coverage_pattern_message,
                radials,
            })
        })
        .collect();

    let mut decoded = Vec::with_capacity(results.len());
    for result in results {
        decoded.push(result?);
    }

    decoded.sort_by_key(|record| record.index);

    Ok(decoded)
}

#[cfg(all(
    test,
    feature = "nexrad-model",
    any(feature = "parallel-decompress", feature = "parallel-decode")
))]
mod tests {
    use super::{scan_parallel, scan_serial, File};

    const TEST_NEXRAD_FILE: &[u8] = include_bytes!("../../../downloads/KDMX20220305_232324_V06");

    #[test]
    fn parallel_scan_matches_serial() {
        let volume = File::new(TEST_NEXRAD_FILE.to_vec());
        let serial = match scan_serial(&volume) {
            Ok(scan) => scan,
            Err(err) => panic!("serial scan: {err}"),
        };
        let parallel = match scan_parallel(&volume) {
            Ok(scan) => scan,
            Err(err) => panic!("parallel scan: {err}"),
        };

        assert_eq!(serial, parallel);
    }
}

impl Debug for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("File");
        debug.field("data.len()", &self.data().len());
        debug.field("header", &self.header());

        #[cfg(feature = "nexrad-model")]
        debug.field(
            "records.len()",
            &self.records().map(|records| records.len()),
        );

        debug.finish()
    }
}
