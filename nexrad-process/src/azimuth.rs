//! Azimuth adjacency shared by the polar algorithms.

use nexrad_model::data::SweepField;

/// Whether `field`'s azimuths cover the full circle: the gap across north between the last
/// and first azimuths is at most twice the azimuth spacing.
pub(crate) fn covers_full_circle(field: &SweepField) -> bool {
    let angles = field.azimuth_angles_degrees();
    let (Some(&first), Some(&last)) = (angles.first(), angles.last()) else {
        return false;
    };
    let wrap_gap = 360.0 - (f64::from(last) - f64::from(first));
    wrap_gap <= 2.0 * f64::from(field.azimuth_spacing_degrees())
}

/// The azimuth index `offset` rows from `az_idx`, wrapping across north when `wraps` and
/// `None` past a sector edge otherwise.
pub(crate) fn neighbor_azimuth(
    az_idx: usize,
    offset: isize,
    az_count: usize,
    wraps: bool,
) -> Option<usize> {
    let neighbor = az_idx as isize + offset;
    if wraps {
        Some(neighbor.rem_euclid(az_count as isize) as usize)
    } else {
        usize::try_from(neighbor).ok().filter(|&n| n < az_count)
    }
}
