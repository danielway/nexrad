//! The decode code enums and `VcpNumber` serialize exactly as their `nexrad-model` counterparts.

#![cfg(all(feature = "serde", feature = "model"))]

use nexrad_decode::messages::digital_radar_data::RadialStatus;
use nexrad_decode::messages::volume_coverage_pattern::{
    ChannelConfiguration, PulseWidth, WaveformType,
};
use nexrad_decode::messages::VcpNumber;
use nexrad_model::data as model;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Asserts that `decoded` and its model conversion serialize identically, and that each form
/// deserializes into the other.
fn assert_parity<D, M>(decoded: D)
where
    D: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug + Copy + Into<M> + From<M>,
    M: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug + Copy,
{
    let as_model: M = decoded.into();
    let decoded_json = serde_json::to_value(decoded).unwrap();
    let model_json = serde_json::to_value(as_model).unwrap();
    assert_eq!(decoded_json, model_json, "{decoded:?}");
    assert_eq!(serde_json::from_value::<M>(decoded_json).unwrap(), as_model);
    assert_eq!(serde_json::from_value::<D>(model_json).unwrap(), decoded);
}

#[test]
fn code_enums_serialize_as_the_model() {
    for code in 0..=u8::MAX {
        assert_parity::<_, model::WaveformType>(WaveformType::from_number(code));
        assert_parity::<_, model::ChannelConfiguration>(ChannelConfiguration::from_number(code));
        assert_parity::<_, model::PulseWidth>(PulseWidth::from_number(code));
        assert_parity::<_, model::RadialStatus>(RadialStatus::from_number(code));
    }
}

#[test]
fn vcp_number_serializes_as_the_bare_number() {
    for number in [0, 35, 212, u16::MAX] {
        assert_parity::<_, model::VcpNumber>(VcpNumber::new(number));
        assert_eq!(
            serde_json::to_value(VcpNumber::new(number)).unwrap(),
            serde_json::json!(number)
        );
    }
}
