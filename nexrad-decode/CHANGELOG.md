# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-10-27

Changes since `1.0.0-rc.3`, the last release candidate.

See [Migrating to 1.0](https://github.com/danielway/nexrad/blob/main/docs/migrating-to-1.0.md) for step-by-step upgrade instructions.

### Added

- `Error` and `Result` are re-exported at the crate root; the `result` module remains. (#158)
- `messages::VcpNumber`, one VCP number type for every message type. (#163)
- `rda_status_data::VcpSelection` and `VcpSource`, returned by `rda_status_data::Message::vcp_number()`. (#163)
- `MessageHeader::parse`, `MessageHeader::SIZE`, `MessageHeader::into_owned()` and accessors for every header field.
  (#166, #173)
- `MessageType::number()` and `MessageType::from_number(u8)`. (#166)
- `MessageType::ModelData` (29), the Archive II ICD's Model Data message, which was `Reserved(29)`; its contents
  decode as `MessageContents::Other`. (#206)
- `clutter_censor_zones::Region<'a>` with typed accessors, and `clutter_censor_zones::OperatorSelectCode`. (#166)
- Module `model` (feature `model`) with `ScanBuilder`, which assembles decoded messages into a `Scan`; all
  protocol-to-model conversion now lives in this crate. (#169)
- `volume_coverage_pattern::Message::to_model()`, `digital_radar_data::Message::site()`, and `From` impls from the
  decode `PulseWidth`, `ChannelConfiguration`, `WaveformType` and `RadialStatus` to the model's (feature `model`).
  (#169, #173)
- `Error::MissingCoveragePatternNumber` (feature `model`), from `ScanBuilder::build` when there was no type 5 message
  and no radial carried a non-zero coverage pattern number. (#169, #190, #208)
- `Error::NegativeFirstGateRange` (feature `model`), for a legacy (type 1) radial whose first gate is at a negative
  range with a zero gate interval. (#208)
- `moment_data()` and `into_moment_data()` on a generic data block's `DataBlock` (feature `model`), taking the product
  from the block's data name. (#170)
- `DataBlockName::product()` (feature `model`). (#173)
- Unit-suffixed accessors where only a raw or `uom` form existed, such as `RadialDataBlock::unambiguous_range_km()`.
  (#173)
- `Error::Model(nexrad_model::Error)` (feature `model`) for decoded values that fail a model constructor's validation;
  it displays only its layer, with the model error as its `source()`. (#185, #195)
- `messages::VcpNumber` constants for VCPs 11, 21, 121, 211 and 221, matching `nexrad_model::data::VcpNumber`. (#188,
  #205)
- `from_number(u8)`, `number()` and `Display` on `volume_coverage_pattern::{WaveformType, ChannelConfiguration,
  PulseWidth}` and `digital_radar_data::RadialStatus`, and `WaveformType::abbreviation()` (the ICD's `CS`, `CD/W`,
  `CD/WO` or `B`). (#205)
- `From` conversions from the model's `WaveformType`, `ChannelConfiguration`, `PulseWidth`, `RadialStatus` and
  `VcpNumber` to decode's, and from decode's `VcpNumber` to the model's (feature `model`). (#205)
- Feature `serde`, which serializes `WaveformType`, `ChannelConfiguration`, `PulseWidth`, `RadialStatus` and
  `VcpNumber` exactly as their `nexrad-model` counterparts. (#205)
- `iter_messages(data)` and `MessageIter`, which yield one `Result` per message and stop after the first error;
  `MessageIter::offset()` reports where. (#190)
- Crate-root re-exports of `decode_messages`, `iter_messages`, `Message`, `MessageContents`, `MessageIter` and
  `MessageType`. (#190)
- `Message::into_owned()`, `MessageContents::into_owned()`, `MessageHeaders::into_owned()` and
  `MessageContents::message_type()`. (#190)
- `Error::SegmentOutOfSequence`, for a missing or out-of-order segment of a multi-segment message. (#190)
- Decoded forms beside the accessors renamed to `_raw`, such as `VolumeDataBlock::zdr_bias_estimate_weighted_mean_db()`.
  (#191)
- `digital_radar_data_legacy::Message` Nyquist velocity and atmospheric attenuation accessors, for header bytes that
  were read as spare. (#191)
- `rda_status_data::RdaBuildNumber::{raw, major, minor}` and `Display`. (#191)
- `digital_radar_data::Message::other_data_blocks()`, the data moment blocks whose names this version does not
  recognise, parsed with the generic data block layout. (#207)

### Changed

- `volume_coverage_pattern::Message::to_model()` returns `Result`, because the model's `ElevationCutBuilder::build`
  and `VolumeCoveragePatternBuilder::build` validate; values decoded from a message always pass, and
  `ScanBuilder::add_message` propagates the error. (#210)
- The minimum supported Rust version is 1.88 (it was declared as 1.75, which was already incorrect). (#157)
- `Error`, `messages::MessageContents`, `messages::MessageType`, `messages::RedundantChannel` and
  `digital_radar_data::DataBlockName` are `#[non_exhaustive]`; matches on them outside this crate need a wildcard
  arm. (#155)
- The `nexrad-model` feature is renamed `model` and optional dependencies are no longer features. (#156)
- Acronyms in type and variant names are UpperCamelCase words (`RdaStatus`, `RdaPrfData`, `OrdaSingleChannel`,
  `CompressedBzip2`), and `WaveformType` variants have descriptive names. (#160)
- `VolumeDataBlock::volume_coverage_pattern()` → `vcp_number()`, returning `VcpNumber`, which decodes VCP 32 instead of
  `Unknown(32)`. (#163, #191)
- `rda_status_data::Message::volume_coverage_pattern()` → `vcp_number()`, returning `Option<VcpSelection>`. (#163,
  #191)
- `MessageHeader` is `MessageHeader<'a>` with private fields read through accessors; `parse` and `SIZE` replace the
  zerocopy reads. (#166, #173)
- `MessageHeaders<'a>` holds owned headers, and `Message::header()` and `contents()` borrow from `self`. (#166)
- `MessageType`'s spare and reserved variants are one `Reserved(u8)`, and it is no longer `#[repr(u8)]`. (#166)
- `clutter_censor_zones::Message::regions()` returns `Region<'a>` values with accessors instead of public wire fields.
  (#166)
- No public type implements `zerocopy` traits. (#166)
- Accessors on decoded messages follow one naming convention ("Accessor Naming" in the crate docs): plain numbers carry
  a unit suffix, `<stem>_raw()` replaces `raw_<stem>()`, and decoded enums take the bare stem.
  (#173)
- `Debug` output prints the new names (#160, #173, #191) and `fnv1a: 0x...` hashes for binary blobs (#162), and shows a
  `DataBlock`'s input `offset` and the `message_type` that records a console or loopback message's direction (#190).
- `digital_radar_data::Message::site()` returns `Result<Option<Site>>`, failing with `Error::Model`, which
  `model::ScanBuilder::add_message` propagates. (#185)
- Moment and radial conversions return `Error::Model` for data the model rejects. (#185)
- `messages::VcpNumber` is a `u16` newtype with `new`, `number()`, `From` both ways, `Ord` by number and associated
  constants (`VcpNumber::VCP_212`) in place of the `Vcp212` and `Unknown(n)` variants; constants work as `match`
  patterns, and a constant added later cannot change how an existing value matches. (#188, #205)
- `digital_radar_data::RadialStatus::VolumeScanStart` and `VolumeScanEnd` are `ScanStart` and `ScanEnd`, the names
  `nexrad_model::data::RadialStatus` uses. (#205)
- The conversions to the model's `WaveformType`, `ChannelConfiguration` and `PulseWidth` keep the code of an `Unknown`
  value. (#205)
- `MessageType` orders by `number()` instead of by declaration order. (#205)
- Converted radials carry the Nyquist velocity and unambiguous range (type 31) or the unambiguous range (type 1), and
  the `Radial` `Debug` output gains both fields. (#188)
- `decode_messages` is strict: it returns the first decoding error instead of skipping the message. An all-zero tail
  shorter than 28 bytes still ends decoding cleanly; any other short tail is `Error::UnexpectedEof`. (#190)
- `MessageContents::Other { message_type, data }` carries the message type and undecoded payload, and a legacy
  (pre-ORDA) clutter filter map (type 15) decodes as `Other`. (#190)
- A frame with the unassigned message type 0 decodes as its own `Other` message. (#190)
- `Error` variants carry structured, input-relative detail, `Error` implements `Clone` and `PartialEq`, and messages no
  longer mention a file. (#190)
- `digital_radar_data::DataBlock` no longer implements `Deref`; use `inner()` or `into_inner()`. (#190)
- The accessor naming convention applies across the crate: `_raw` for wire encodings, bitfields and codes, one
  `vcp_number` stem, `<thing>_count()` counts, `*_segment_number()`, `is_multi_segment()`, byte strings as
  `<stem>_raw()` beside `Option<&str>`, `alarm::Definition`, `digital_radar_data::SpotBlankingFlags`, and `MessageType`
  variants named like their `MessageContents` variants. (#191)
- Code enums carry an unlisted wire value in one `Unknown(<wire type>)` variant; `SpotBlankingStatus` no longer
  truncates it, and transition power source code 4 is `Unknown(4)`. (#191)
- `rda_status_data::RdaBuildNumber` is a `const`-constructible newtype over the wire value, ordered by build, instead of
  an enum of known builds; builds outside 12.0–24.0 are no longer `Unknown`. (#191)
- VCP elevation, sector edge and EBC angles and `azimuth_indexing_mode_degrees` are `f32`, matching the model. (#191)
- Legacy radial conversion sets the Nyquist velocity (`None` when zero), and the legacy message's `Debug` output gains
  its Nyquist velocity and atmospheric attenuation. (#191)
- A legacy (pre-ORDA) clutter filter bypass map (type 13) decodes as `Other` instead of an empty map. (#191)
- docs.rs shows the feature each feature-gated item requires. (#196)
- The published crate leaves out the tests and benchmarks that need files from outside it, and the fuzz targets. (#196)
- Documentation only: the README states the supported data scope and its code blocks compile as doc tests; the type 3
  and type 18 accessor names are the ICD table mnemonics and are fixed for 1.x. (#209)
- An unrecognised type 31 data block no longer fails the decode: a `'D'` block is kept in `other_data_blocks()`
  and an `'R'` block is skipped, so `File::scan` reads volumes from a build that adds a moment.
  `Error::UnknownDataBlockType` is returned only for a block whose type byte is neither `'R'` nor `'D'`. (#207)
- The code enums with an `Unknown(n)` or `Reserved(n)` variant in the message modules are `#[non_exhaustive]`; an
  `Unknown(n)` value may become a named variant in a minor release. (#207, #215)
- `model::ScanBuilder::build` no longer requires a type 5 message. Without one the scan's `coverage_pattern()` is
  `None`; its `coverage_pattern_number()` is the type 5 message's number when non-zero, and otherwise the first
  non-zero number in a type 1 header or type 31 volume data block. (#208)
- `WaveformType`, `ChannelConfiguration`, `PulseWidth` and `RadialStatus` are `#[non_exhaustive]` like the other code
  enums; a variant only a newer `nexrad-model` names converts to `Unknown(255)`. (#215)
- The struct-like variants of `Error` are `#[non_exhaustive]`: patterns outside this crate need `..`. (#215)

### Removed

- `digital_radar_data::VolumeCoveragePattern`; use `messages::VcpNumber`. (#163)
- `rda_status_data::VolumeCoveragePatternNumber`; use `VcpSelection` and `VcpSource`. (#163)
- `digital_radar_data::DataMomentPointer`, `DataMomentPointerType` and `DataMomentGenericPointerType`. (#166)
- Module `summarize`; iterate the decoded messages instead (`nexrad-data/examples/realtime.rs` has an example). (#168)
- `Error::Io` and `From<std::io::Error> for Error`, which nothing produced. (#168)
- `GenericDataBlock::{moment_data, into_moment_data, cfp_moment_data, into_cfp_moment_data}`; call `moment_data()` on
  the `DataBlock` instead. (#170)
- With `uom`, `digital_radar_data::Header::azimuth_resolution_spacing() -> Angle`; use
  `azimuth_resolution_spacing_degrees()`. (#173)
- The `sha2` and `hex` dependencies. (#162)
- The `uom` feature, the `uom` dependency and every accessor returning a `uom` quantity: `MessageHeader::{message_size,
  segment_size}`, `digital_radar_data::Header::{azimuth_angle, elevation_angle, azimuth_indexing_mode, radial_length}`,
  `lrtup` on `VolumeDataBlock`, `ElevationDataBlock` and `RadialDataBlock`, `VolumeDataBlock::{latitude, longitude,
  site_height, tower_height, horizontal_shv_tx_power, vertical_shv_tx_power, initial_system_differential_phase}`,
  `RadialDataBlock::{unambiguous_range, nyquist_velocity}`, `GenericDataBlockHeader::{data_moment_range,
  data_moment_range_sample_interval, moment_size}`, `digital_radar_data_legacy::Message::nyquist_velocity` and
  `clutter_filter_map::RangeZone::end_range`. `uom` is pre-1.0 and would tie this crate's major version to its own;
  each has a unit-suffixed sibling (`azimuth_angle_degrees()`, `unambiguous_range_km()`, `lrtup_bytes()`, ...).
  Unit-typed accessors may come later as a separate crate or extension trait. (#202)
- `Error::Decoding`, `Error::InvalidMessageLength` and `Error::DataSpansSegmentBoundary`, which nothing constructed.
  (#190)
- `rda_adaptation_data::Message`'s raw readers (`raw_data`, `read_*_at`, `total_size`); use the typed accessors. (#190)
- `GenericDataBlockHeader::reserved()`, `VolumeDataBlock::spare()` and the `is_legacy()` methods; `lrtup_bytes()`
  distinguishes the layouts. (#190)
- The `log` dependency. (#190)
- `digital_radar_data::DataBlockId::data_block_name_str()` and `data_block_type_char()`. (#191)
- `rda_status_data::RdaBuildNumber::as_float()` and `is_known()`. (#191)

### Fixed

- VCP elevation, sector edge and EBC angles with the sign bit set decode as negative instead of wrapped to 360 degrees.
  (#146)
- `DataBlockId::data_block_name()` returns `DataBlockName::SpectrumWidth` for the `"SW "` block. (#173)
- A legacy (type 1) message with an unknown radial status converts to `RadialStatus::Unknown(v)`. (#173)
- Multi-segment messages read each segment's full payload, so clutter filter bypass maps (type 13) decode, clutter
  filter maps (type 15) have 360 azimuth segments, and RDA adaptation data (type 18) fields past the first segment are
  read from the right offsets. (#190)
- A digital radar data (type 31) message that pads past its last data block, as TDWR radials do, no longer throws
  off the messages after it; decoding resumes at the message's declared end.
- Legacy (type 1) radial conversion no longer mirrors a negative range to the first gate: the type 1 Doppler first gate
  at -375 m was converted as +375 m, which moved every velocity and spectrum width gate 750 m outward. Leading gates at
  a negative range are dropped and the moment starts at the first gate at a non-negative range (125 m, with 250 m
  gates). (#208)
