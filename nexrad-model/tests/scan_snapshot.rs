#![cfg(feature = "serde")]

use insta::assert_yaml_snapshot;
use nexrad_data::volume;
use sha2::{Digest, Sha256};

const TEST_NEXRAD_FILE: &[u8] = include_bytes!("KDMX20220305_232324_V06");

#[test]
fn test_decode_map_scan() {
  let volume = volume::File::new(TEST_NEXRAD_FILE.to_vec());
  let scan = volume.scan().expect("decodes into scan model");
  assert_yaml_snapshot!(scan, {
    ".**.values" => insta::dynamic_redaction(|value, _path| redact_bytes_like(value)),
  });
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
