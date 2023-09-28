//!
//! Provides utilities like [decode_file] for decoding NEXRAD data.
//!

use std::io::{Cursor, Read, Seek, SeekFrom};
use std::mem::size_of;

use bincode::{DefaultOptions, Options};
use serde::de::DeserializeOwned;

use crate::model::{
    DataBlockHeader, DataFile, DataMoment, ElevationData, GenericData, Message31, Message31Header,
    MessageHeader, RadialData, VolumeData, VolumeHeaderRecord,
};
use crate::result::Result;

/// Given an uncompressed data file, decodes it and returns the decoded structure.
pub fn decode_file(data: &Vec<u8>) -> Result<DataFile> {
    let mut reader = Cursor::new(data);

    let file_header: VolumeHeaderRecord = decode_file_header(&mut reader)?;
    let mut file = DataFile::new(file_header);

    while reader.position() < data.len() as u64 {
        let message_header: MessageHeader = deserialize(&mut reader)?;

        if message_header.msg_type() == 31 {
            decode_message_31(&mut reader, &mut file)?;
        } else {
            let ff_distance = 2432 - size_of::<MessageHeader>();
            reader.seek(SeekFrom::Current(ff_distance as i64))?;
        }
    }

    Ok(file)
}

fn decode_file_header<R: Read + Seek>(reader: &mut R) -> Result<VolumeHeaderRecord> {
    Ok(deserialize(reader)?)
}

fn decode_message_31(reader: &mut Cursor<&Vec<u8>>, file: &mut DataFile) -> Result<()> {
    let start_pos = reader.position();

    let message_31_header: Message31Header = deserialize(reader)?;
    let mut message = Message31::new(message_31_header);

    let pointers_space = message.header().data_block_count() as usize * size_of::<u32>();
    let mut pointers_raw = vec![0; pointers_space];
    reader.read_exact(&mut pointers_raw).unwrap();

    let data_block_pointers = pointers_raw
        .chunks_exact(size_of::<u32>())
        .map(|v| <u32>::from_be_bytes(v.try_into().unwrap()))
        .collect::<Vec<_>>();

    for pointer in data_block_pointers {
        if pointer as u64 != reader.position() {
            reader.seek(SeekFrom::Start(start_pos + pointer as u64))?;
        }

        let data_block: DataBlockHeader = deserialize(reader)?;
        reader.seek(SeekFrom::Current(-4))?;

        let data_block_name = String::from_utf8_lossy(data_block.data_name()).to_string();
        match data_block_name.as_str() {
            "VOL" => {
                let data: VolumeData = deserialize(reader)?;
                message.set_volume_data(data);

                // todo: I'm missing 8 bytes here
                // reader.seek(SeekFrom::Current(8))?;
            }
            "ELV" => {
                let data: ElevationData = deserialize(reader)?;
                message.set_elevation_data(data);
            }
            "RAD" => {
                let data: RadialData = deserialize(reader)?;
                message.set_radial_data(data);
            }
            "REF" | "VEL" | "CFP" | "SW " | "ZDR" | "PHI" | "RHO" => {
                let generic_data: GenericData = deserialize(reader)?;

                let mut moment_data = vec![0; generic_data.moment_size()];
                reader.read_exact(&mut moment_data).unwrap();

                let data = DataMoment::new(generic_data, moment_data);
                match data_block_name.as_str() {
                    "REF" => message.set_reflectivity_data(data),
                    "VEL" => message.set_velocity_data(data),
                    "SW " => message.set_sw_data(data),
                    "ZDR" => message.set_zdr_data(data),
                    "PHI" => message.set_phi_data(data),
                    "RHO" => message.set_rho_data(data),
                    "CFP" => message.set_cfp_data(data),
                    _ => panic!("Unexpected generic data block name: {}", data_block_name),
                }
            }
            _ => panic!("Unknown data block name: {}", data_block_name),
        }
    }

    let elevation_scans = file.elevation_scans_mut();
    if !elevation_scans.contains_key(&message.header().elev_num()) {
        elevation_scans.insert(message.header().elev_num(), vec![message]);
    } else {
        elevation_scans
            .get_mut(&message.header().elev_num())
            .unwrap()
            .push(message);
    }

    Ok(())
}

/// Attempts to deserialize some struct from the provided binary reader.
fn deserialize<R: Read + Seek, S: DeserializeOwned>(reader: &mut R) -> Result<S> {
    Ok(DefaultOptions::new()
        .with_fixint_encoding()
        .with_big_endian()
        .deserialize_from(reader.by_ref())?)
}
