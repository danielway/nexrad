use std::io::Cursor;

use insta::{assert_yaml_snapshot, internals::Content};
use nexrad_data::volume;
use nexrad_decode::messages::decode_messages;
use sha2::{Digest, Sha256};

const TEST_NEXRAD_FILE: &[u8] = include_bytes!("KDMX20220305_232324_V06");

#[test]
fn test_decode_volume() {
    let volume = volume::File::new(TEST_NEXRAD_FILE.to_vec());
    for (record_number, mut record) in volume.records().into_iter().enumerate() {
        if record.compressed() {
            record = record.decompress().expect("decompresses records");
        }

        let mut reader = Cursor::new(record.data());
        let messages = decode_messages(&mut reader).expect("decodes successfully");

        assert_yaml_snapshot!(format!("Record {}", record_number), messages, {
          ".**.rpg_unknown" => insta::dynamic_redaction(|value, _path| redact_bytes_like(value)),
          ".**.encoded_data" => insta::dynamic_redaction(|value, _path| redact_bytes_like(value)),
        });
    }
}

fn redact_bytes_like(value: insta::internals::Content) -> insta::internals::Content {
    if let Some(items) = value.as_slice() {
        let mut buf = Vec::with_capacity(items.len());
        for item in items {
            if let Some(n) = item.as_u64() {
                if n <= 255 {
                    buf.push(n as u8);
                    continue;
                }
            }
            panic!("unexpected value in binary data: {:?}", item);
        }
        return hash_bytes_to_content(&buf);
    }
    value
}

fn hash_bytes_to_content(bytes: &[u8]) -> insta::internals::Content {
    let mut hash = Sha256::new();
    hash.update(bytes);

    let digest = hex::encode(hash.finalize());
    format!("bytes::<{}>::sha256:{digest}", bytes.len()).into()
}
