//!
//! Message type 31 "Digital Radar Data" consists of base data information such as reflectivity,
//! mean radial velocity, spectrum width, differential reflectivity, differential phase, correlation
//! coefficient, azimuth angle, elevation angle, cut type, scanning strategy, and calibration
//! parameters. The frequency and volume of the message is dependent on the scanning strategy and
//! the type of data associated with that strategy.
//!

mod header;
pub use header::Header;

mod message;
pub use message::Message;

mod data_block_id;
pub use data_block_id::DataBlockId;

mod volume_data_block;
pub use volume_data_block::VolumeDataBlock;

mod generic_data_block;
pub use generic_data_block::{GenericDataBlock, GenericDataBlockHeader};

mod elevation_data_block;
pub use elevation_data_block::ElevationDataBlock;

mod radial_data_block;
pub use radial_data_block::RadialDataBlock;

mod definitions;
pub use definitions::*;

mod spot_blanking_status;
pub use spot_blanking_status::*;

mod pointers;
pub use pointers::*;
