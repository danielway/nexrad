/// Splits compressed LDM record data into individual records.
pub fn split_record_data(data: &Vec<u8>) -> Vec<&[u8]> {
    let mut compressed_records = Vec::new();

    let mut position = 0;
    loop {
        if position >= data.len() {
            break;
        }

        let mut record_size = [0; 4];
        record_size.copy_from_slice(&data[position..position + 4]);
        let record_size = i32::from_be_bytes(record_size).abs();

        let whole_record_size = record_size as usize + 4;
        compressed_records.push(&data[position..position + whole_record_size]);
        position += whole_record_size;
    }

    compressed_records
}
