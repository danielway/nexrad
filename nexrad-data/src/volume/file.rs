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
    ///
    /// Returns an error if the record data is truncated or malformed.
    pub fn records(&self) -> crate::result::Result<Vec<Record<'_>>> {
        split_compressed_records(&self.0[size_of::<Header>()..])
    }

    /// Decodes this volume file into a common model scan containing sweeps and radials with moment
    /// data.
    #[cfg(feature = "nexrad-model")]
    pub fn scan(&self) -> crate::result::Result<nexrad_model::data::Scan> {
        use crate::result::Error;
        use nexrad_decode::messages::volume_coverage_pattern as vcp;
        use nexrad_decode::messages::MessageContents;
        use nexrad_model::data::{
            ChannelConfiguration, ElevationCut, PulseWidth, Radial, Scan, Sweep,
            VolumeCoveragePattern, WaveformType,
        };

        let records = self.records()?;

        let process_record = |record: Record<'_>| -> crate::result::Result<(
            Option<vcp::Message<'static>>,
            Vec<Radial>,
        )> {
            let record = if record.compressed() {
                record.decompress()?
            } else {
                Record::new(record.data().to_vec())
            };

            let mut vcp = None;
            let mut radials = Vec::new();
            for message in record.messages()? {
                match message.into_contents() {
                    MessageContents::DigitalRadarData(m) => radials.push(m.into_radial()?),
                    MessageContents::VolumeCoveragePattern(m) => {
                        if vcp.is_none() {
                            vcp = Some(m.into_owned());
                        }
                    }
                    _ => {}
                }
            }
            Ok((vcp, radials))
        };

        #[cfg(feature = "parallel")]
        let results: Vec<_> = {
            use rayon::prelude::*;
            records
                .into_par_iter()
                .map(process_record)
                .collect::<crate::result::Result<Vec<_>>>()?
        };

        #[cfg(not(feature = "parallel"))]
        let results: Vec<_> = records
            .into_iter()
            .map(process_record)
            .collect::<crate::result::Result<Vec<_>>>()?;

        let mut coverage_pattern_message = None;
        let mut radials = Vec::new();
        for (vcp, r) in results {
            if coverage_pattern_message.is_none() {
                coverage_pattern_message = vcp;
            }
            radials.extend(r);
        }

        // Convert the VCP message to the model representation
        let vcp_msg = coverage_pattern_message.ok_or(Error::MissingCoveragePattern)?;
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

        let coverage_pattern = VolumeCoveragePattern::new(
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
        );

        Ok(Scan::new(coverage_pattern, Sweep::from_radials(radials)))
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
