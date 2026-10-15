use super::raw;
use super::{ElevationDataBlock, Header};
use crate::result::Result;
use crate::segmented_slice_reader::SegmentedSliceReader;

/// The volume coverage pattern message describes the current scan pattern. This is sent on
/// wideband connection and the start of each volume scan.
///
/// This message's contents correspond to ICD 2620002AA section 3.2.4.12 Table XI.
#[derive(Debug, Clone, PartialEq)]
pub struct Message<'a> {
    /// The decoded volume coverage pattern header.
    header: Header<'a>,

    /// The decoded elevation data blocks.
    elevations: Vec<ElevationDataBlock<'a>>,
}

impl<'a> Message<'a> {
    /// Parse a volume coverage pattern message from the input.
    pub(crate) fn parse(reader: &mut SegmentedSliceReader<'a, '_>) -> Result<Self> {
        let raw_header = reader.take_ref::<raw::Header>()?;

        let elevation_cuts = raw_header.elevation_cut_count.get() as usize;
        let raw_elevations = reader.take_slice::<raw::ElevationDataBlock>(elevation_cuts)?;

        let header = Header::new(raw_header);
        let elevations = raw_elevations.iter().map(ElevationDataBlock::new).collect();

        Ok(Self { header, elevations })
    }

    /// Convert this message to an owned version with `'static` lifetime.
    pub fn into_owned(self) -> Message<'static> {
        Message {
            header: self.header.into_owned(),
            elevations: self
                .elevations
                .into_iter()
                .map(ElevationDataBlock::into_owned)
                .collect(),
        }
    }

    /// The decoded volume coverage pattern header.
    pub fn header(&self) -> &Header<'a> {
        &self.header
    }

    /// The decoded elevation data blocks.
    pub fn elevations(&self) -> &[ElevationDataBlock<'a>] {
        &self.elevations
    }

    /// Convert this message into the common model's volume coverage pattern, including every
    /// elevation cut.
    #[cfg(feature = "model")]
    pub fn to_model(&self) -> nexrad_model::data::VolumeCoveragePattern {
        use nexrad_model::data::{ElevationCut, VcpNumber, VolumeCoveragePattern};

        let header = &self.header;
        let elevation_cuts = self
            .elevations
            .iter()
            .map(|elev| {
                ElevationCut::builder(
                    elev.elevation_angle_degrees(),
                    elev.waveform_type().into(),
                    elev.channel_configuration().into(),
                )
                .azimuth_rate_degrees_per_second(elev.azimuth_rate_degrees_per_second())
                .super_resolution_half_degree_azimuth(elev.super_resolution_half_degree_azimuth())
                .super_resolution_quarter_km_reflectivity(
                    elev.super_resolution_quarter_km_reflectivity(),
                )
                .super_resolution_doppler_to_300km(elev.super_resolution_doppler_to_300km())
                .super_resolution_dual_pol_to_300km(elev.super_resolution_dual_pol_to_300km())
                .surveillance_prf_number(elev.surveillance_prf_number())
                .surveillance_prf_pulse_count(elev.surveillance_prf_pulse_count_radial())
                .reflectivity_threshold_db(elev.reflectivity_threshold_db())
                .velocity_threshold_db(elev.velocity_threshold_db())
                .spectrum_width_threshold_db(elev.spectrum_width_threshold_db())
                .differential_reflectivity_threshold_db(
                    elev.differential_reflectivity_threshold_db(),
                )
                .differential_phase_threshold_db(elev.differential_phase_threshold_db())
                .correlation_coefficient_threshold_db(elev.correlation_coefficient_threshold_db())
                .is_sails_cut(elev.is_sails_cut())
                .sails_sequence_number(elev.sails_sequence_number())
                .is_mrle_cut(elev.is_mrle_cut())
                .mrle_sequence_number(elev.mrle_sequence_number())
                .is_mpda_cut(elev.is_mpda_cut())
                .is_base_tilt_cut(elev.is_base_tilt_cut())
                .build()
            })
            .collect();

        VolumeCoveragePattern::builder(VcpNumber::from_number(header.vcp_number_raw()))
            .version(header.version())
            .doppler_velocity_resolution_mps(header.doppler_velocity_resolution_mps())
            .pulse_width(header.pulse_width().into())
            .sails_enabled(header.is_sails_vcp())
            .sails_cut_count(header.sails_cut_count())
            .mrle_enabled(header.is_mrle_vcp())
            .mrle_cut_count(header.mrle_cut_count())
            .mpda_enabled(header.is_mpda_vcp())
            .base_tilt_enabled(header.is_base_tilt_vcp())
            .base_tilt_count(header.base_tilt_count())
            .sequence_active(header.vcp_sequencing_sequence_active())
            .truncated(header.vcp_sequencing_truncated())
            .elevation_cuts(elevation_cuts)
            .build()
    }
}
