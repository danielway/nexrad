//!
//! This module contains models representing digital radar data collected by the NEXRAD weather
//! radar network. These models and their APIs are intended to be ergonomic, understandable, and
//! performant. They do not exactly match the encoded structure from common archival formats.
//!
//! todo: note optional uom interfaces
//! todo: note serialization support
//!

/// A single radar scan composed of a series of sweeps. This represents a single volume scan which
/// is composed of multiple sweeps at different elevations. The pattern of sweeps, including
/// elevations and resolution, is determined by the scanning strategy of the radar. This is
/// referred to as the Volume Coverage Pattern.
pub struct Scan;

/// A single radar sweep composed of a series of radials. This represents a full rotation of the
/// radar at some elevation angle and contains the Level II data (reflectivity, velocity, and
/// spectrum width) for each azimuth angle in that sweep. The resolution of the sweep dictates the
/// azimuthal distance between rays and thus and number of rays in the sweep. Multiple sweeps are
/// taken at different elevation angles to create a volume scan.
pub struct Sweep;

/// A single radar ray composed of a series of gates. This represents a single azimuth angle and
/// elevation angle pair at a point in time and contains the Level II data (reflectivity, velocity,
/// and spectrum width) for each range gate in that ray. The range of the radar and gate interval
/// distance determines the resolution of the ray and the number of gates in the ray.
pub struct Radial;
