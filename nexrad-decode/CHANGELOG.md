# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Changes since `1.0.0-rc.3`, the last published release. Together they make up the 1.0.0 release.

See [Migrating to 1.0](https://github.com/danielway/nexrad/blob/main/docs/migrating-to-1.0.md) for step-by-step upgrade instructions.

### Added

- `Error` and `Result` are re-exported at the crate root (`nexrad_decode::Error`); `nexrad_decode::result` remains
  the defining module. (#158)
- `messages::VcpNumber`, one VCP number enum for every message type. (#163)
- `rda_status_data::VcpSelection` and `rda_status_data::VcpSource`, returned by
  `rda_status_data::Message::volume_coverage_pattern()`. (#163)
- `MessageHeader::parse(bytes)`, `MessageHeader::SIZE`, `MessageHeader::into_owned()` and accessors for every header
  field (`segment_size_raw`, `redundant_channel_raw`, `message_type_raw`, `sequence_number`, `date_time`,
  `segment_count`, `segment_number`, `message_size_bytes`, `segment_size_bytes`). (#166, #173)
- `MessageType::number()` and `MessageType::from_number(u8)`. (#166)
- `clutter_censor_zones::Region<'a>` with typed accessors, and `clutter_censor_zones::OperatorSelectCode`. (#166)
- Module `model` (feature `model`) with `ScanBuilder` (`new`, `add_message`, `add_messages`, `merge`, `build`), which
  assembles decoded messages into a `nexrad_model::data::Scan`. All protocol-to-model conversion now lives in this
  crate. (#169)
- `volume_coverage_pattern::Message::to_model()`, `digital_radar_data::Message::site()`, and `From` impls from the
  decode `PulseWidth`, `ChannelConfiguration`, `WaveformType` and `digital_radar_data::RadialStatus` to their
  `nexrad_model::data` counterparts (feature `model`). (#169, #173)
- `Error::MissingCoveragePattern`, returned by `ScanBuilder::build` when no type 5 message was seen. (#169)
- `DataBlock<'a, GenericDataBlock<'a>>::moment_data()` and `into_moment_data()` (feature `model`), which take the
  product from the block's data name; call them on the `DataBlock` returned by `Message::*_data_block()`. (#170)
- `DataBlockName::product()` (feature `model`). (#173)
- New unit-suffixed accessors where only a raw or `uom` form existed, including
  `RadialDataBlock::unambiguous_range_km()` and `nyquist_velocity_mps()`, `GenericDataBlockHeader::tover_db()`,
  `snr_threshold_db()` and `moment_size_bytes()`, `Header::azimuth_resolution_spacing_degrees()` and
  `rda_status_data::Message::vertical_reflectivity_calibration_correction_db()`. (#173)
- `Error::Model(nexrad_model::Error)` (feature `model`), returned when decoded values fail a model constructor's
  validation, such as moment data whose gate count does not match its data or a radial with an azimuth outside
  `0.0..360.0`. (#185)
- `messages::VcpNumber::{Vcp11, Vcp21, Vcp121, Vcp211, Vcp221}`, matching `nexrad_model::data::VcpNumber` variant for
  variant. (#188)
- `iter_messages(data)` and `MessageIter`, which decode one message per step, yield a `Result` per message and stop
  after the first error; `MessageIter::offset()` reports where decoding stopped. (#190)
- `decode_messages`, `iter_messages`, `Message`, `MessageContents`, `MessageIter` and `MessageType` are re-exported at
  the crate root; the `messages` paths remain. (#190)
- `Message::into_owned()`, `MessageContents::into_owned()` and `MessageHeaders::into_owned()`, which copy a decoded
  message out of the input buffer as `Message<'static>`, and `MessageContents::message_type()`. (#190)
- `Error::SegmentOutOfSequence`, returned when a segment of a multi-segment message is missing or out of order. (#190)

### Changed

- The minimum supported Rust version is now 1.88 (it was declared as 1.75, which was already incorrect). (#157)
- `Error`, `messages::MessageContents`, `messages::MessageType`, `messages::RedundantChannel` and
  `digital_radar_data::DataBlockName` are `#[non_exhaustive]`; matches on them outside this crate need a wildcard
  arm. (#155)
- The `nexrad-model` feature is renamed to `model`, and `uom` is no longer a default feature. Optional dependencies use
  `dep:` syntax, so their names are no longer features. (#156)
- Acronyms in names follow UpperCamelCase (#160). The full table is in the PR #160 description; in summary:
  - `rda_status_data::RDAStatus` → `RdaStatus`, `RDABuildNumber` → `RdaBuildNumber`,
    `RMSControlStatus` → `RmsControlStatus` (variants `NonRms`, `RmsInControl`, `RdaInControl`).
  - `MessageType` and `MessageContents` variants `RDA*`/`RPG*` → `Rda*`/`Rpg*` (e.g. `RDAStatusData` →
    `RdaStatusData`, `RDAPRFData` → `RdaPrfData`, `RPGVolumeCoveragePattern` → `RpgVolumeCoveragePattern`).
  - `RedundantChannel::ORDA*` → `Orda*`; `CompressionIndicator::CompressedBZIP2`/`CompressedZLIB` →
    `CompressedBzip2`/`CompressedZlib`; `ProcessingStatus::RxRNoise`/`CBT` → `RxrNoise`/`Cbt`;
    `RadialStatus::ElevationStartVCPFinal` → `ElevationStartVcpFinal`;
    `CommandAcknowledgement::RemoteVCPReceived` → `RemoteVcpReceived`; `TransitionPowerSourceStatus::OK` → `Ok`.
  - `ChannelConfiguration::SZ2Phase` → `Sz2Phase`, and the `WaveformType` variants `CS`, `CDW`, `CDWO`, `B`, `SPP` →
    `ContiguousSurveillance`, `ContiguousDopplerWithAmbiguityResolution`,
    `ContiguousDopplerWithoutAmbiguityResolution`, `Batch`, `StaggeredPulsePair`.
- `digital_radar_data::VolumeDataBlock::volume_coverage_pattern()` returns `VcpNumber`. (#163)
- VCP 32 decodes as `VcpNumber::Vcp32`; the old `digital_radar_data::VolumeCoveragePattern` had no variant for it and
  returned `Unknown(32)`. (#163)
- `rda_status_data::Message::volume_coverage_pattern()` returns `Option<VcpSelection>`: `number()` returns `VcpNumber`
  instead of `i16`, and `source()` replaces `local()`/`remote()`. (#163)
- `MessageHeader` is now `MessageHeader<'a>` with private fields read through accessors: `segment_size` →
  `segment_size_raw()`, `redundant_channel` → `redundant_channel_raw()` or `redundant_channel()`, `message_type` →
  `message_type_raw()` or `message_type()`, `sequence_number` → `sequence_number()`, `date`/`time` → `date_time()`,
  `segment_count`/`segment_number` → `segment_count()`/`segment_number()`. Replace
  `MessageHeader::ref_from_prefix(bytes)` with `MessageHeader::parse(bytes)` and `size_of::<MessageHeader>()` with
  `MessageHeader::SIZE`. (#166, #173)
- `MessageHeaders<'a>` holds owned headers (`Single(MessageHeader<'a>)`, `Multiple(Vec<MessageHeader<'a>>)`);
  `MessageHeaders::primary()`, `iter()` and `Message::header()` return headers borrowed from `self`, and
  `Message::contents()` returns `&MessageContents<'a>`. (#166)
- `MessageType` variants `Spare1`, `Reserved1`..`Reserved5` and the FAA RMS reserved variants are replaced by
  `Reserved(u8)`. `MessageType` is no longer `#[repr(u8)]`: replace `ty as u8` with `ty.number()`. Its derived `Ord`
  now places reserved types after `RdaLogData`. (#166)
- `clutter_censor_zones::Message::regions()` returns `&[clutter_censor_zones::Region<'a>]` with accessors
  (`start_range_km()`, `stop_range_km()`, `start_azimuth_degrees()`, `stop_azimuth_degrees()`,
  `elevation_segment_number()`, `operator_select_code()`, `operator_select_code_raw()`) instead of public wire
  fields. (#166)
- No public type implements `zerocopy` traits any more. (#166)
- Accessors on decoded messages follow one naming convention (see "Accessor Naming" in the crate docs); the full
  old → new table is in the PR #173 description (#173):
  - `MessageHeader`: `rda_redundant_channel()` → `redundant_channel()`.
  - `digital_radar_data::Header`: `*_raw()` angle and length accessors become `azimuth_angle_degrees()`,
    `elevation_angle_degrees()`, `radial_length_bytes()`; `azimuth_resolution_spacing_type()` →
    `azimuth_resolution_spacing()`.
  - `digital_radar_data::VolumeDataBlock`: `*_raw()` and unsuffixed numeric accessors gain unit suffixes
    (`latitude_degrees()`, `site_height_meters()`, `calibration_constant_db()`, `horizontal_shv_tx_power_kw()`, ...);
    `volume_coverage_pattern_number()` → `volume_coverage_pattern_raw()`.
  - `digital_radar_data::RadialDataBlock` and `ElevationDataBlock`: `lrtup_raw()` → `lrtup_bytes()`, and noise,
    calibration and attenuation accessors gain `_dbm`, `_db` and `_db_per_km` suffixes.
  - `digital_radar_data_legacy::Message`: angles, gate ranges, intervals and resolutions gain unit suffixes;
    `elevation_number()` returns `u8`; `radial_status()` returns `digital_radar_data::RadialStatus` and
    `vcp_number()` returns `VcpNumber`, with the codes available as `radial_status_raw()` and `vcp_number_raw()`.
  - `volume_coverage_pattern::Header` and `ElevationDataBlock`: `message_size()` → `message_size_halfwords()`, and
    angle, rate, velocity-resolution and threshold accessors gain `_degrees`, `_degrees_per_second`, `_mps` and `_db`
    suffixes.
  - `clutter_filter_map::RangeZone`: `raw_op_code()` → `op_code_raw()`, `end_range()` → `end_range_km()`, and the uom
    `end_range_uom()` → `end_range()`.
  - `request_for_data::Message`: `raw_data_request_type()` → `data_request_type_raw()`.
  - `rda_status_data::Message`: every `raw_<stem>()` → `<stem>_raw()` (and `raw_rda_build_number()` →
    `build_number_raw()`); `average_transmitter_power()` → `average_transmitter_power_watts()`;
    `horizontal_reflectivity_calibration_correction()` → `horizontal_reflectivity_calibration_correction_db()`.
  - `console_message`, `loopback_test`, `rda_log_data` and `rda_adaptation_data`: `message_size()` →
    `message_size_bytes()` / `message_size_halfwords()`, `compressed_size()`/`decompressed_size()` →
    `*_size_bytes()`, `raw_data()` → `data_raw()`.
- `Debug` output changed: renamed types and variants print their new names (#160), binary blobs print
  `fnv1a: 0x<16 hex digits>` instead of `sha256: "<hex>"` (#162), and `digital_radar_data_legacy::Message` field
  names follow the accessor names (#173).
- `digital_radar_data::Message::site()` returns `Result<Option<Site>>`, failing with `Error::Model` when the radar
  identifier or location is invalid; `model::ScanBuilder::add_message` propagates it. (#185)
- `DataBlock::moment_data`/`into_moment_data`, `Message::radial`/`into_radial` (types 1 and 31) return
  `Error::Model` for data the model rejects instead of building an invalid `MomentData` or `Radial`. (#185)
- VCP numbers 11, 21, 121, 211 and 221 decode as their named `VcpNumber` variants instead of `Unknown(n)`. (#188)
- `digital_radar_data::Message::radial`/`into_radial` set the model radial's Nyquist velocity and unambiguous range
  from the radial data block, and `digital_radar_data_legacy::Message::radial`/`into_radial` set its unambiguous range
  from the header; each stays `None` when absent or zero. The `Radial` `Debug` output gains both fields. (#188)
- `decode_messages` is strict: it returns the error for the first message it cannot decode (a truncated message, a bad
  data block pointer, an unknown data block name, a missing segment) instead of logging it and skipping the message.
  A tail of fewer than 28 bytes ends decoding cleanly when every byte is zero (Archive II frames are zero-padded); any
  other short tail is `Error::UnexpectedEof`. Use `iter_messages` to keep the messages before a failure. (#190)
- `MessageContents::Other` is `Other { message_type, data }`, carrying the message's type and its undecoded payload.
  A clutter filter map (type 15) from a legacy (pre-ORDA) RDA uses the legacy notch width layout and decodes as
  `Other`. (#190)
- A fixed-segment frame with the unassigned message type 0, an unused slot in the Archive II metadata record, decodes
  as its own `Other` message whatever its segment fields say. (#190)
- `Error` variants carry structured, input-relative detail: `UnexpectedEof { offset }`,
  `UnknownDataBlockType { block_type: [u8; 3], offset }` and `MessageMissingDate { message_type }`;
  `InvalidDataBlockPointer` and `DataSpansSegmentBoundary` report offsets into the input slice. `Error` implements
  `Clone` and `PartialEq`, `MissingCoveragePattern` requires the `model` feature, and messages no longer mention a
  file. (#190)
- `digital_radar_data::DataBlock` no longer implements `Deref`; call `inner()` or `into_inner()` to reach the block.
  (#190)
- `Debug` output of `digital_radar_data::DataBlock` gains the block's input `offset`, and that of
  `console_message::Message` and `loopback_test::Message` gains the `message_type` that records their direction.
  (#190)

### Removed

- `digital_radar_data::VolumeCoveragePattern`; use `messages::VcpNumber`. (#163)
- `rda_status_data::VolumeCoveragePatternNumber`; use `rda_status_data::VcpSelection` and `VcpSource`. (#163)
- `digital_radar_data::DataMomentPointer`, `DataMomentPointerType` and `DataMomentGenericPointerType`, which nothing
  used. (#166)
- Module `summarize` and all of its items (`messages`, `MessageSummary`, `MessageGroupSummary`, `VcpInfo`,
  `VcpElevationInfo`, `extract_vcp_info`, `RdaStatusInfo`, `extract_rda_status_info`). Iterate the decoded messages
  and match on `message.contents()` instead; `nexrad-data/examples/realtime.rs` has a minimal example. (#168)
- `Error::Io` and `From<std::io::Error> for Error`, which nothing in the crate produced; handle I/O errors at the I/O
  layer. (#168)
- `GenericDataBlock::{moment_data, into_moment_data, cfp_moment_data, into_cfp_moment_data}`; call `moment_data()` /
  `into_moment_data()` on the `DataBlock` wrapper instead. (#170)
- With `uom`, `digital_radar_data::Header::azimuth_resolution_spacing() -> Angle`; use
  `azimuth_resolution_spacing_degrees()`. (#173)
- The `sha2` and `hex` dependencies. (#162)
- `Error::Decoding(String)` and `Error::InvalidMessageLength`, which nothing constructed. (#190)
- `rda_adaptation_data::Message::{read_integer4_at, read_real4_at, read_string_at, data_raw, total_size}`; use the
  typed accessors. (#190)
- `digital_radar_data::GenericDataBlockHeader::reserved()`, `VolumeDataBlock::spare()` and `is_legacy()`, and
  `RadialDataBlock::is_legacy()`; `lrtup_bytes()` distinguishes the block layouts. (#190)
- The `log` dependency. (#190)

### Fixed

- Angles with the sign bit set decode as negative: `volume_coverage_pattern::ElevationDataBlock` elevation, sector edge
  and EBC angles for negative elevation cuts were returned as their 360-degree-wrapped value (for example 359.8 instead
  of -0.2). (#146)
- `DataBlockId::data_block_name()` returns `DataBlockName::SpectrumWidth` for the `"SW "` block instead of
  `Unknown(*b"SW ")`. (#173)
- With `uom`, `RadialDataBlock::unambiguous_range()` applies the ICD 0.1 km scale; it was 10× too large. (#173)
- With `uom`, `MessageHeader::segment_size()` no longer overflows `u16` for segments over 32767 halfwords. (#173)
- Converting a legacy (type 1) message with an unknown radial status yields `RadialStatus::Unknown(v)` instead of
  `IntermediateRadialData`. (#173)
- Multi-segment messages read each segment's full payload. The decoder sized payloads 12 bytes short, so clutter filter
  bypass maps (type 13) failed to decode and were dropped, clutter filter maps (type 15) read 350 azimuth segments
  instead of 360, and RDA adaptation data (type 18) fields beyond the first segment, such as the site name, were read
  from the wrong offsets. (#190)
