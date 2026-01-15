//! Debug test for KCRP file.

use nexrad_data::volume;

const KCRP_FILE: &[u8] = include_bytes!("../../downloads/KCRP20170826_044114_V06");
const KDMX_FILE: &[u8] = include_bytes!("../../downloads/KDMX20220305_232324_V06");

#[test]
fn debug_kcrp_messages() {
    println!("Hello, world");
    let volume = volume::File::new(KCRP_FILE.to_vec());

    // Search through all records to find one with message type 31
    for (record_idx, mut record) in volume.records().into_iter().enumerate() {
        if record.compressed() {
            record = record.decompress().expect("decompresses");
        }

        let data = record.data();

        // Find message type 31
        let mut offset = 0;
        let mut msg_idx = 0;
        while offset + 28 <= data.len() {
            let segment_size = u16::from_be_bytes([data[offset + 12], data[offset + 13]]);
            let message_type = data[offset + 15];

            if message_type == 31 {
                println!(
                    "Found message type 31 in record {}, message {}, offset={}",
                    record_idx, msg_idx, offset
                );

                let header_start = offset + 28; // After MessageHeader

                // Print radar identifier
                let radar_id = &data[header_start..header_start + 4];
                println!("  Radar ID: {:?}", String::from_utf8_lossy(radar_id));

                // data_block_count is at offset 30 from header_start
                let count_offset = header_start + 30;
                let data_block_count =
                    u16::from_be_bytes([data[count_offset], data[count_offset + 1]]);
                println!("  data_block_count: {}", data_block_count);

                // Print pointers
                let pointers_start = header_start + 32;
                let current_pos_after_ptrs = pointers_start + (data_block_count as usize) * 4;
                let relative_pos_after_ptrs = current_pos_after_ptrs - header_start;
                println!(
                    "  After reading pointers: position={}, relative_position={}",
                    current_pos_after_ptrs, relative_pos_after_ptrs
                );

                for j in 0..data_block_count.min(10) {
                    let ptr_offset = pointers_start + j as usize * 4;
                    let pointer = u32::from_be_bytes([
                        data[ptr_offset],
                        data[ptr_offset + 1],
                        data[ptr_offset + 2],
                        data[ptr_offset + 3],
                    ]);
                    let diff: i64 = pointer as i64 - relative_pos_after_ptrs as i64;
                    println!(
                        "  Pointer[{}]: {} (diff from current: {})",
                        j, pointer, diff
                    );
                }

                return; // Found it, stop
            }

            // Advance to next message
            if segment_size == 65535 {
                let segment_count = u16::from_be_bytes([data[offset + 22], data[offset + 23]]);
                let msg_size = ((segment_count as u32) << 16) | ((segment_size as u32) << 1);
                offset += msg_size as usize;
            } else {
                offset += 2432;
            }
            msg_idx += 1;
        }
    }

    println!("No message type 31 found!");
}

#[test]
fn debug_kdmx_messages() {
    let volume = volume::File::new(KDMX_FILE.to_vec());

    // Search through all records to find one with message type 31
    for (record_idx, mut record) in volume.records().into_iter().enumerate() {
        if record.compressed() {
            record = record.decompress().expect("decompresses");
        }

        let data = record.data();

        // Find message type 31
        let mut offset = 0;
        let mut msg_idx = 0;
        while offset + 28 <= data.len() {
            let segment_size = u16::from_be_bytes([data[offset + 12], data[offset + 13]]);
            let message_type = data[offset + 15];

            if message_type == 31 {
                println!(
                    "Found message type 31 in record {}, message {}, offset={}",
                    record_idx, msg_idx, offset
                );

                let header_start = offset + 28; // After MessageHeader

                // Print radar identifier
                let radar_id = &data[header_start..header_start + 4];
                println!("  Radar ID: {:?}", String::from_utf8_lossy(radar_id));

                // data_block_count is at offset 30 from header_start
                let count_offset = header_start + 30;
                let data_block_count =
                    u16::from_be_bytes([data[count_offset], data[count_offset + 1]]);
                println!("  data_block_count: {}", data_block_count);

                // Print pointers
                let pointers_start = header_start + 32;
                let current_pos_after_ptrs = pointers_start + (data_block_count as usize) * 4;
                let relative_pos_after_ptrs = current_pos_after_ptrs - header_start;
                println!(
                    "  After reading pointers: position={}, relative_position={}",
                    current_pos_after_ptrs, relative_pos_after_ptrs
                );

                for j in 0..data_block_count.min(10) {
                    let ptr_offset = pointers_start + j as usize * 4;
                    let pointer = u32::from_be_bytes([
                        data[ptr_offset],
                        data[ptr_offset + 1],
                        data[ptr_offset + 2],
                        data[ptr_offset + 3],
                    ]);
                    let diff: i64 = pointer as i64 - relative_pos_after_ptrs as i64;
                    println!(
                        "  Pointer[{}]: {} (diff from current: {})",
                        j, pointer, diff
                    );
                }

                return; // Found it, stop
            }

            // Advance to next message
            if segment_size == 65535 {
                let segment_count = u16::from_be_bytes([data[offset + 22], data[offset + 23]]);
                let msg_size = ((segment_count as u32) << 16) | ((segment_size as u32) << 1);
                offset += msg_size as usize;
            } else {
                offset += 2432;
            }
            msg_idx += 1;
        }
    }

    println!("No message type 31 found!");
}
