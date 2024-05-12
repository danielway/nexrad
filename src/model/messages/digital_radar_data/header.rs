use crate::model::messages::digital_radar_data::pointers::DataMomentPointer;
use crate::model::messages::digital_radar_data::spot_blanking_status::SpotBlankingStatus;
use crate::model::messages::digital_radar_data::{
    CompressionIndicator, DataMomentGenericPointerType, DataMomentPointerType, RadialStatus,
};
use crate::model::messages::primitive_aliases::{
    Code1, Integer1, Integer2, Integer4, Real4, ScaledInteger1,
};
use crate::model::util::get_datetime;
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use std::fmt::Debug;
use uom::si::angle::degree;
use uom::si::f64::{Angle, Information};
use uom::si::information::byte;

/// The digital radar data message header block precedes base data information for a particular
/// radial and includes parameters for that radial and information about the following data blocks.
#[derive(Deserialize)]
pub struct Header {
    /// ICAO radar identifier.
    pub radar_identifier: [u8; 4],

    /// Collection time in milliseconds past midnight, GMT.
    pub time: Integer4,

    /// This message's date represented as a count of days since 1 January 1970 00:00 GMT. It is
    /// also referred-to as a "modified Julian date" where it is the Julian date - 2440586.5.
    pub date: Integer2,

    /// Radial number within the elevation scan. These range up to 720, in 0.5 degree increments.
    pub azimuth_number: Integer2,

    /// Azimuth angle at which the radial was collected in degrees.
    pub azimuth_angle: Real4,

    /// Indicates if the message is compressed and what type of compression was used. This header is
    /// not compressed.
    ///
    /// Values:
    ///   0 = Uncompressed
    ///   1 = Compressed using BZIP2
    ///   2 = Compressed using ZLIB
    ///   3 = Future use
    pub compression_indicator: Code1,

    /// Spare to force halfword alignment.
    pub spare: u8,

    /// Uncompressed length of the radial in bytes (including the data header block).
    pub radial_length: Integer2,

    /// Azimuthal spacing between adjacent radials. Note this is the commanded value, not
    /// necessarily the actual spacing.
    ///
    /// Values:
    ///   1 = 0.5 degrees
    ///   2 = 1.0 degrees
    pub azimuth_resolution_spacing: Code1,

    /// The radial's status within the larger scan (e.g. first, last).
    ///
    /// Statuses:
    ///   0 = Start of elevation
    ///   1 = Intermediate radial data
    ///   2 = End of elevation
    ///   3 = Start of volume scan
    ///   4 = End of volume scan
    ///   5 = Start of new elevation which is the last in the VCP
    pub radial_status: Code1,

    /// The radial's elevation number within the volume scan.
    pub elevation_number: Integer1,

    /// The sector number within cut. A value of 0 is only valid for continuous surveillance cuts.
    pub cut_sector_number: Integer1,

    /// The radial's collection elevation angle.
    pub elevation_angle: Real4,

    /// The spot blanking status for the current radial, elevation, and volume scan.
    ///
    /// Statuses:
    ///   0 = None
    ///   1 = Radial
    ///   2 = Elevation
    ///   4 = Volume
    pub radial_spot_blanking_status: Code1,

    /// The azimuth indexing value (if keyed to constant angles).
    ///
    /// Values:
    ///   0     = No indexing
    ///   1-100 = Indexing angle of 0.01 to 1.00 degrees
    pub azimuth_indexing_mode: ScaledInteger1,

    /// The number of "data moment" blocks following this header block, from 4 to 10. There are
    /// always volume, elevation, and radial information blocks and a reflectivity data moment
    /// block. The following 6 data moment blocks are optional, depending on scanning mode. The next
    /// 10 fields on this header contain pointers to each block, if available in the message.
    pub data_block_count: Integer2,

    /// Pointer to the volume data block.
    pub volume_data_block_pointer: Integer4,

    /// Pointer to the elevation data block.
    pub elevation_data_block_pointer: Integer4,

    /// Pointer to the radial data block.
    pub radial_data_block_pointer: Integer4,

    /// Pointer to the reflectivity ("REF") data moment block.
    pub reflectivity_data_moment_block_pointer: Integer4,

    /// Pointer to the velocity ("VEL") data moment block.
    pub velocity_data_moment_block_pointer: Integer4,

    /// Pointer to the spectrum width ("SW") data moment block.
    pub spectrum_width_data_moment_block_pointer: Integer4,

    /// Pointer to the differential reflectivity ("ZDR") data moment block.
    pub differential_reflectivity_data_moment_block_pointer: Integer4,

    /// Pointer to the differential phase ("PHI") data moment block.
    pub differential_phase_data_moment_block_pointer: Integer4,

    /// Pointer to the correlation coefficient ("RHO") data moment block.
    pub correlation_coefficient_data_moment_block_pointer: Integer4,

    /// Pointer to the specific differential phase ("CFP") data moment block.
    pub specific_diff_phase_data_moment_block_pointer: Integer4,
}

impl Header {
    /// ICAO radar identifier.
    pub fn radar_identifier(&self) -> String {
        String::from_utf8_lossy(&self.radar_identifier).to_string()
    }

    /// The collection date and time for this data.
    pub fn date_time(&self) -> DateTime<Utc> {
        get_datetime(self.date, Duration::milliseconds(self.time as i64))
    }

    /// Azimuth angle at which the radial was collected.
    pub fn azimuth_angle(&self) -> Angle {
        Angle::new::<degree>(self.azimuth_angle as f64)
    }

    /// Whether the message is compressed and what type of compression was used.
    pub fn compression_indicator(&self) -> CompressionIndicator {
        match self.compression_indicator {
            0 => CompressionIndicator::Uncompressed,
            1 => CompressionIndicator::CompressedBZIP2,
            2 => CompressionIndicator::CompressedZLIB,
            _ => CompressionIndicator::FutureUse,
        }
    }

    /// Uncompressed length of the radial (including the data header block).
    pub fn radial_length(&self) -> Information {
        Information::new::<byte>(self.radial_length as f64)
    }

    /// Azimuthal spacing between adjacent radials.
    pub fn azimuth_resolution_spacing(&self) -> Angle {
        Angle::new::<degree>(self.azimuth_resolution_spacing as f64 * 0.5)
    }

    /// The radial's status within the larger scan.
    pub fn radial_status(&self) -> RadialStatus {
        match self.radial_status {
            0 => RadialStatus::ElevationStart,
            1 => RadialStatus::IntermediateRadialData,
            2 => RadialStatus::ElevationEnd,
            3 => RadialStatus::VolumeScanStart,
            4 => RadialStatus::VolumeScanEnd,
            _ => RadialStatus::ElevationStartVCPFinal,
        }
    }

    /// The radial's collection elevation angle.
    pub fn elevation_angle(&self) -> Angle {
        Angle::new::<degree>(self.elevation_angle as f64)
    }

    /// The spot blanking status for the current radial, elevation, and volume scan.
    pub fn radial_spot_blanking_status(&self) -> SpotBlankingStatus {
        SpotBlankingStatus::new(self.radial_spot_blanking_status)
    }

    /// The azimuth indexing value (if keyed to constant angles).
    pub fn azimuth_indexing_module(&self) -> Option<Angle> {
        if self.azimuth_indexing_mode == 0 {
            None
        } else {
            Some(Angle::new::<degree>(
                self.azimuth_indexing_mode as f64 * 0.01,
            ))
        }
    }

    /// Returns a list of pointers to the data blocks in this message
    pub fn pointers(&self) -> Vec<DataMomentPointer> {
        let mut pointers = Vec::with_capacity(10);

        pointers.push(DataMomentPointer {
            pointer: self.volume_data_block_pointer,
            data_moment_type: DataMomentPointerType::Volume,
        });

        pointers.push(DataMomentPointer {
            pointer: self.elevation_data_block_pointer,
            data_moment_type: DataMomentPointerType::Elevation,
        });

        pointers.push(DataMomentPointer {
            pointer: self.radial_data_block_pointer,
            data_moment_type: DataMomentPointerType::Radial,
        });

        pointers.push(DataMomentPointer {
            pointer: self.reflectivity_data_moment_block_pointer,
            data_moment_type: DataMomentPointerType::Generic(
                DataMomentGenericPointerType::Reflectivity,
            ),
        });

        if self.data_block_count >= 5 {
            pointers.push(DataMomentPointer {
                pointer: self.velocity_data_moment_block_pointer,
                data_moment_type: DataMomentPointerType::Generic(
                    DataMomentGenericPointerType::Velocity,
                ),
            });
        }

        if self.data_block_count >= 6 {
            pointers.push(DataMomentPointer {
                pointer: self.spectrum_width_data_moment_block_pointer,
                data_moment_type: DataMomentPointerType::Generic(
                    DataMomentGenericPointerType::SpectrumWidth,
                ),
            });
        }

        if self.data_block_count >= 7 {
            pointers.push(DataMomentPointer {
                pointer: self.differential_reflectivity_data_moment_block_pointer,
                data_moment_type: DataMomentPointerType::Generic(
                    DataMomentGenericPointerType::DifferentialReflectivity,
                ),
            });
        }

        if self.data_block_count >= 8 {
            pointers.push(DataMomentPointer {
                pointer: self.differential_phase_data_moment_block_pointer,
                data_moment_type: DataMomentPointerType::Generic(
                    DataMomentGenericPointerType::DifferentialPhase,
                ),
            });
        }

        if self.data_block_count >= 9 {
            pointers.push(DataMomentPointer {
                pointer: self.correlation_coefficient_data_moment_block_pointer,
                data_moment_type: DataMomentPointerType::Generic(
                    DataMomentGenericPointerType::CorrelationCoefficient,
                ),
            });
        }

        if self.data_block_count >= 10 {
            pointers.push(DataMomentPointer {
                pointer: self.specific_diff_phase_data_moment_block_pointer,
                data_moment_type: DataMomentPointerType::Generic(
                    DataMomentGenericPointerType::SpecificDiffPhase,
                ),
            });
        }

        pointers
    }
}

impl Debug for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataHeaderBlock")
            .field("radar_identifier", &self.radar_identifier())
            .field("date_time", &self.date_time())
            .field("azimuth_number", &self.azimuth_number)
            .field("azimuth_angle", &self.azimuth_angle())
            .field("compression_indicator", &self.compression_indicator())
            .field("radial_length", &self.radial_length())
            .field(
                "azimuth_resolution_spacing",
                &self.azimuth_resolution_spacing(),
            )
            .field("radial_status", &self.radial_status())
            .field("elevation_number", &self.elevation_number)
            .field("cut_sector_number", &self.cut_sector_number)
            .field("elevation_angle", &self.elevation_angle())
            .field(
                "radial_spot_blanking_status",
                &self.radial_spot_blanking_status(),
            )
            .field("azimuth_indexing_mode", &self.azimuth_indexing_module())
            .field("data_block_count", &self.data_block_count)
            .field("volume_data_block_pointer", &self.volume_data_block_pointer)
            .field(
                "elevation_data_block_pointer",
                &self.elevation_data_block_pointer,
            )
            .field("radial_data_block_pointer", &self.radial_data_block_pointer)
            .field(
                "reflectivity_data_moment_block_pointer",
                &self.reflectivity_data_moment_block_pointer,
            )
            .field(
                "velocity_data_moment_block_pointer",
                &self.velocity_data_moment_block_pointer,
            )
            .field(
                "spectrum_width_data_moment_block_pointer",
                &self.spectrum_width_data_moment_block_pointer,
            )
            .field(
                "differential_reflectivity_data_moment_block_pointer",
                &self.differential_reflectivity_data_moment_block_pointer,
            )
            .field(
                "differential_phase_data_moment_block_pointer",
                &self.differential_phase_data_moment_block_pointer,
            )
            .field(
                "correlation_coefficient_data_moment_block_pointer",
                &self.correlation_coefficient_data_moment_block_pointer,
            )
            .field(
                "specific_diff_phase_data_moment_block_pointer",
                &self.specific_diff_phase_data_moment_block_pointer,
            )
            .finish()
    }
}
