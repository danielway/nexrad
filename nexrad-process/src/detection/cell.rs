use std::collections::VecDeque;

use crate::result::{Error, Result};
use nexrad_model::data::{GateStatus, SweepField};
use nexrad_model::geo::{GeoPoint, PolarPoint, RadarCoordinateSystem};

/// Geographic bounding box of a storm cell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StormCellBounds {
    /// Minimum latitude (southern edge).
    min_latitude: f64,
    /// Maximum latitude (northern edge).
    max_latitude: f64,
    /// Minimum longitude (western edge).
    min_longitude: f64,
    /// Maximum longitude (eastern edge).
    max_longitude: f64,
}

impl StormCellBounds {
    /// Minimum latitude (southern edge).
    pub fn min_latitude(&self) -> f64 {
        self.min_latitude
    }

    /// Maximum latitude (northern edge).
    pub fn max_latitude(&self) -> f64 {
        self.max_latitude
    }

    /// Minimum longitude (western edge).
    pub fn min_longitude(&self) -> f64 {
        self.min_longitude
    }

    /// Maximum longitude (eastern edge).
    pub fn max_longitude(&self) -> f64 {
        self.max_longitude
    }
}

/// A detected storm cell identified by connected-component analysis on a
/// reflectivity sweep field.
///
/// Each cell represents a contiguous region of radar gates above a configured
/// reflectivity threshold. Geographic properties are computed using the radar
/// coordinate system.
#[derive(Debug, Clone, PartialEq)]
pub struct StormCell {
    /// Unique identifier for this cell within the detection run (0-based).
    id: u32,
    /// Geographic centroid of the cell, computed as the reflectivity-weighted
    /// mean of all gate positions.
    centroid: GeoPoint,
    /// Maximum reflectivity value (dBZ) observed in this cell.
    max_reflectivity_dbz: f32,
    /// Mean reflectivity value (dBZ) across all gates in this cell.
    mean_reflectivity_dbz: f32,
    /// Number of polar gates comprising this cell.
    gate_count: usize,
    /// Approximate area of the cell in square kilometers.
    area_km2: f64,
    /// Geographic bounding box of the cell.
    bounds: StormCellBounds,
    /// Elevation angle of the sweep this cell was detected on (degrees).
    elevation_degrees: f32,
    /// Azimuth of the gate with maximum reflectivity (degrees).
    max_reflectivity_azimuth_degrees: f32,
    /// Range of the gate with maximum reflectivity (km).
    max_reflectivity_range_km: f64,
}

impl StormCell {
    /// Unique identifier for this cell within the detection run.
    ///
    /// Cells are sorted by descending maximum reflectivity, so ID 0 is the
    /// strongest cell.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Geographic centroid of the cell (reflectivity-weighted).
    pub fn centroid(&self) -> GeoPoint {
        self.centroid
    }

    /// Maximum reflectivity value in dBZ.
    pub fn max_reflectivity_dbz(&self) -> f32 {
        self.max_reflectivity_dbz
    }

    /// Mean reflectivity value in dBZ.
    pub fn mean_reflectivity_dbz(&self) -> f32 {
        self.mean_reflectivity_dbz
    }

    /// Number of polar gates in this cell.
    pub fn gate_count(&self) -> usize {
        self.gate_count
    }

    /// Approximate area in square kilometers.
    pub fn area_km2(&self) -> f64 {
        self.area_km2
    }

    /// Geographic bounding box.
    pub fn bounds(&self) -> &StormCellBounds {
        &self.bounds
    }

    /// Elevation angle of the source sweep in degrees.
    pub fn elevation_degrees(&self) -> f32 {
        self.elevation_degrees
    }

    /// Azimuth of the maximum reflectivity gate (degrees).
    pub fn max_reflectivity_azimuth_degrees(&self) -> f32 {
        self.max_reflectivity_azimuth_degrees
    }

    /// Range of the maximum reflectivity gate (km).
    pub fn max_reflectivity_range_km(&self) -> f64 {
        self.max_reflectivity_range_km
    }
}

/// Detects storm cells in a reflectivity sweep field using connected-component
/// labeling.
///
/// The algorithm identifies contiguous regions of gates with reflectivity at or
/// above a configurable threshold, then computes geographic properties for each
/// region using the radar coordinate system.
///
/// # Algorithm
///
/// 1. Scan all gates in the polar grid. Mark each gate as "above threshold" if
///    its status is `Valid` and its value >= `reflectivity_threshold_dbz`.
/// 2. Perform flood-fill connected-component labeling on the above-threshold
///    gates. Two gates are connected if they are adjacent in azimuth or range
///    (4-connectivity). Azimuth wraps: the last azimuth index is adjacent to
///    the first.
/// 3. Filter out components with fewer than `min_gate_count` gates.
/// 4. For each remaining component, compute cell properties.
///
/// # Example
///
/// ```ignore
/// use nexrad_process::detection::StormCellDetector;
/// use nexrad_model::geo::RadarCoordinateSystem;
///
/// let detector = StormCellDetector::new(35.0, 10)?;
/// let cells = detector.detect(&reflectivity_field, &coord_system)?;
///
/// for cell in &cells {
///     println!("Cell {} at ({:.2}, {:.2}): max {:.1} dBZ, area {:.1} km²",
///         cell.id(),
///         cell.centroid().latitude,
///         cell.centroid().longitude,
///         cell.max_reflectivity_dbz(),
///         cell.area_km2(),
///     );
/// }
/// ```
pub struct StormCellDetector {
    /// Minimum reflectivity (dBZ) for a gate to be included in a cell.
    reflectivity_threshold_dbz: f32,
    /// Minimum number of gates for a component to be retained as a cell.
    min_gate_count: usize,
}

impl StormCellDetector {
    /// Create a new storm cell detector.
    ///
    /// # Parameters
    ///
    /// - `reflectivity_threshold_dbz` — Minimum reflectivity in dBZ. Gates
    ///   at or above this value are candidates for cell membership.
    /// - `min_gate_count` — Minimum number of contiguous gates for a region
    ///   to be considered a storm cell. Regions smaller than this are
    ///   discarded as noise.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidParameter`] if `min_gate_count` is zero.
    pub fn new(reflectivity_threshold_dbz: f32, min_gate_count: usize) -> Result<Self> {
        if min_gate_count == 0 {
            return Err(Error::InvalidParameter(
                "min_gate_count must be >= 1".to_string(),
            ));
        }
        Ok(Self {
            reflectivity_threshold_dbz,
            min_gate_count,
        })
    }

    /// Detect storm cells in the given reflectivity sweep field.
    ///
    /// # Parameters
    ///
    /// - `field` — A reflectivity sweep field (dBZ values).
    /// - `coord_system` — The radar coordinate system for computing geographic
    ///   positions.
    ///
    /// # Returns
    ///
    /// A vector of detected storm cells, sorted by descending maximum
    /// reflectivity. Returns an empty vector if no cells are found.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidGeometry`] if the field has zero azimuths or
    /// zero gates.
    pub fn detect(
        &self,
        field: &SweepField,
        coord_system: &RadarCoordinateSystem,
    ) -> Result<Vec<StormCell>> {
        let az_count = field.azimuth_count();
        let num_gates = field.gate_count();

        if az_count == 0 {
            return Err(Error::InvalidGeometry(
                "field has zero azimuths".to_string(),
            ));
        }
        if num_gates == 0 {
            return Err(Error::InvalidGeometry("field has zero gates".to_string()));
        }

        let total = az_count * num_gates;

        // Phase 1: Build threshold mask
        let mut above_threshold = vec![false; total];
        for az_idx in 0..az_count {
            for gate_idx in 0..num_gates {
                let (val, status) = field.get(az_idx, gate_idx);
                if status == GateStatus::Valid && val >= self.reflectivity_threshold_dbz {
                    above_threshold[az_idx * num_gates + gate_idx] = true;
                }
            }
        }

        // Phase 2: Flood-fill connected-component labeling
        let mut visited = vec![false; total];
        let mut components: Vec<Vec<(usize, usize)>> = Vec::new();

        for az_idx in 0..az_count {
            for gate_idx in 0..num_gates {
                let idx = az_idx * num_gates + gate_idx;
                if !above_threshold[idx] || visited[idx] {
                    continue;
                }

                // BFS flood fill for new component
                let mut component: Vec<(usize, usize)> = Vec::new();
                let mut queue = VecDeque::new();

                visited[idx] = true;
                queue.push_back((az_idx, gate_idx));

                while let Some((az, gate)) = queue.pop_front() {
                    component.push((az, gate));

                    // 4-connected neighbors: azimuth ± 1 (wrapping), range ± 1 (clamped)
                    let az_prev = if az == 0 { az_count - 1 } else { az - 1 };
                    let az_next = if az == az_count - 1 { 0 } else { az + 1 };

                    let neighbors: [(usize, usize); 2] = [(az_prev, gate), (az_next, gate)];

                    for &(n_az, n_gate) in &neighbors {
                        let n_idx = n_az * num_gates + n_gate;
                        if above_threshold[n_idx] && !visited[n_idx] {
                            visited[n_idx] = true;
                            queue.push_back((n_az, n_gate));
                        }
                    }

                    // Range neighbors (no wrapping)
                    if gate > 0 {
                        let n_idx = az * num_gates + (gate - 1);
                        if above_threshold[n_idx] && !visited[n_idx] {
                            visited[n_idx] = true;
                            queue.push_back((az, gate - 1));
                        }
                    }
                    if gate + 1 < num_gates {
                        let n_idx = az * num_gates + (gate + 1);
                        if above_threshold[n_idx] && !visited[n_idx] {
                            visited[n_idx] = true;
                            queue.push_back((az, gate + 1));
                        }
                    }
                }

                components.push(component);
            }
        }

        // Phase 3 & 4: Filter by size and compute properties
        let mut cells: Vec<StormCell> = Vec::new();

        let az_spacing_rad = (field.azimuth_spacing_degrees() as f64).to_radians();

        for component in &components {
            if component.len() < self.min_gate_count {
                continue;
            }

            let cell = self.compute_cell_properties(field, coord_system, component, az_spacing_rad);
            cells.push(cell);
        }

        // Sort by descending max reflectivity
        cells.sort_by(|a, b| {
            b.max_reflectivity_dbz
                .partial_cmp(&a.max_reflectivity_dbz)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Re-assign IDs after sorting so ID 0 = strongest cell
        for (i, cell) in cells.iter_mut().enumerate() {
            cell.id = i as u32;
        }

        Ok(cells)
    }

    /// Compute properties for a single storm cell from its component gates.
    fn compute_cell_properties(
        &self,
        field: &SweepField,
        coord_system: &RadarCoordinateSystem,
        component: &[(usize, usize)],
        az_spacing_rad: f64,
    ) -> StormCell {
        let mut max_dbz = f32::MIN;
        let mut sum_dbz: f64 = 0.0;
        let mut weighted_lat_sum: f64 = 0.0;
        let mut weighted_lon_sum: f64 = 0.0;
        let mut weight_sum: f64 = 0.0;
        let mut min_lat = f64::MAX;
        let mut max_lat = f64::MIN;
        let mut min_lon = f64::MAX;
        let mut max_lon = f64::MIN;
        let mut max_dbz_az: f32 = 0.0;
        let mut max_dbz_range: f64 = 0.0;
        let mut total_area_km2: f64 = 0.0;

        for &(az_idx, gate_idx) in component {
            let (val, _) = field.get(az_idx, gate_idx);

            let azimuth_deg = field.azimuths()[az_idx];
            let range_km =
                field.first_gate_range_km() + gate_idx as f64 * field.gate_interval_km();

            // Reflectivity statistics
            sum_dbz += val as f64;
            if val > max_dbz {
                max_dbz = val;
                max_dbz_az = azimuth_deg;
                max_dbz_range = range_km;
            }

            // Geographic position of this gate
            let geo = coord_system.polar_to_geo(PolarPoint {
                azimuth_degrees: azimuth_deg,
                range_km,
                elevation_degrees: field.elevation_degrees(),
            });

            // Reflectivity-weighted centroid
            let weight = val as f64;
            weighted_lat_sum += geo.latitude * weight;
            weighted_lon_sum += geo.longitude * weight;
            weight_sum += weight;

            // Bounding box
            if geo.latitude < min_lat {
                min_lat = geo.latitude;
            }
            if geo.latitude > max_lat {
                max_lat = geo.latitude;
            }
            if geo.longitude < min_lon {
                min_lon = geo.longitude;
            }
            if geo.longitude > max_lon {
                max_lon = geo.longitude;
            }

            // Gate area: annular sector approximation
            let gate_area = az_spacing_rad * range_km * field.gate_interval_km();
            total_area_km2 += gate_area;
        }

        let centroid = GeoPoint {
            latitude: weighted_lat_sum / weight_sum,
            longitude: weighted_lon_sum / weight_sum,
        };

        StormCell {
            id: 0, // re-assigned after sorting
            centroid,
            max_reflectivity_dbz: max_dbz,
            mean_reflectivity_dbz: (sum_dbz / component.len() as f64) as f32,
            gate_count: component.len(),
            area_km2: total_area_km2,
            bounds: StormCellBounds {
                min_latitude: min_lat,
                max_latitude: max_lat,
                min_longitude: min_lon,
                max_longitude: max_lon,
            },
            elevation_degrees: field.elevation_degrees(),
            max_reflectivity_azimuth_degrees: max_dbz_az,
            max_reflectivity_range_km: max_dbz_range,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexrad_model::meta::Site;

    fn test_site() -> Site {
        Site::new(*b"KTLX", 35.3331, -97.2778, 370, 10)
    }

    fn test_coord_system() -> RadarCoordinateSystem {
        RadarCoordinateSystem::new(&test_site())
    }

    /// Create a 360-azimuth, `gate_count`-gate field with all gates set to NoData.
    fn make_empty_field(gate_count: usize) -> SweepField {
        let azimuths: Vec<f32> = (0..360).map(|i| i as f32).collect();
        SweepField::new_empty("Reflectivity", "dBZ", 0.5, azimuths, 1.0, 2.0, 0.25, gate_count)
    }

    /// Create a uniform field where all gates have the given value.
    fn make_uniform_field(gate_count: usize, value: f32) -> SweepField {
        let mut field = make_empty_field(gate_count);
        for az in 0..360 {
            for gate in 0..gate_count {
                field.set(az, gate, value, GateStatus::Valid);
            }
        }
        field
    }

    #[test]
    fn test_no_cells_below_threshold() {
        let cs = test_coord_system();
        let field = make_uniform_field(100, 20.0);

        let detector = StormCellDetector::new(35.0, 5).unwrap();
        let cells = detector.detect(&field, &cs).unwrap();

        assert!(cells.is_empty());
    }

    #[test]
    fn test_single_cell_detected() {
        let cs = test_coord_system();
        let mut field = make_empty_field(100);

        // Paint a 5x5 block at 40 dBZ
        for az in 10..15 {
            for gate in 20..25 {
                field.set(az, gate, 40.0, GateStatus::Valid);
            }
        }

        let detector = StormCellDetector::new(35.0, 5).unwrap();
        let cells = detector.detect(&field, &cs).unwrap();

        assert_eq!(cells.len(), 1);
        assert_eq!(cells[0].gate_count(), 25);
        assert_eq!(cells[0].max_reflectivity_dbz(), 40.0);
        assert_eq!(cells[0].id(), 0);
    }

    #[test]
    fn test_two_separated_cells() {
        let cs = test_coord_system();
        let mut field = make_empty_field(100);

        // Block A: strong
        for az in 10..15 {
            for gate in 20..25 {
                field.set(az, gate, 50.0, GateStatus::Valid);
            }
        }

        // Block B: weaker, well-separated
        for az in 100..105 {
            for gate in 50..55 {
                field.set(az, gate, 40.0, GateStatus::Valid);
            }
        }

        let detector = StormCellDetector::new(35.0, 5).unwrap();
        let cells = detector.detect(&field, &cs).unwrap();

        assert_eq!(cells.len(), 2);
        // Sorted by descending max reflectivity
        assert!(cells[0].max_reflectivity_dbz() >= cells[1].max_reflectivity_dbz());
        assert_eq!(cells[0].max_reflectivity_dbz(), 50.0);
        assert_eq!(cells[1].max_reflectivity_dbz(), 40.0);
    }

    #[test]
    fn test_min_gate_count_filters_noise() {
        let cs = test_coord_system();
        let mut field = make_empty_field(100);

        // Single gate at 60 dBZ — below the min_gate_count threshold
        field.set(50, 30, 60.0, GateStatus::Valid);

        let detector = StormCellDetector::new(35.0, 5).unwrap();
        let cells = detector.detect(&field, &cs).unwrap();

        assert!(cells.is_empty());
    }

    #[test]
    fn test_azimuth_wrapping() {
        let cs = test_coord_system();
        let mut field = make_empty_field(100);

        // Gates spanning the azimuth wrap: 358, 359, 0, 1, 2
        for &az in &[358, 359, 0, 1, 2] {
            field.set(az, 30, 40.0, GateStatus::Valid);
        }

        let detector = StormCellDetector::new(35.0, 3).unwrap();
        let cells = detector.detect(&field, &cs).unwrap();

        // Should be a single cell, not two
        assert_eq!(cells.len(), 1);
        assert_eq!(cells[0].gate_count(), 5);
    }

    #[test]
    fn test_invalid_min_gate_count_zero() {
        let result = StormCellDetector::new(35.0, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_field_no_error() {
        let cs = test_coord_system();
        let field = make_empty_field(100);

        let detector = StormCellDetector::new(35.0, 5).unwrap();
        let cells = detector.detect(&field, &cs).unwrap();

        assert!(cells.is_empty());
    }

    #[test]
    fn test_cell_centroid_is_geographic() {
        let cs = test_coord_system();
        let mut field = make_empty_field(100);

        // Cell near the radar
        for az in 10..15 {
            for gate in 5..10 {
                field.set(az, gate, 45.0, GateStatus::Valid);
            }
        }

        let detector = StormCellDetector::new(35.0, 5).unwrap();
        let cells = detector.detect(&field, &cs).unwrap();

        assert_eq!(cells.len(), 1);
        let centroid = cells[0].centroid();

        // Centroid should be near the radar site (within a few degrees)
        assert!(
            (centroid.latitude - 35.3331).abs() < 2.0,
            "centroid latitude {} too far from radar",
            centroid.latitude
        );
        assert!(
            (centroid.longitude - (-97.2778)).abs() < 2.0,
            "centroid longitude {} too far from radar",
            centroid.longitude
        );
        assert!(!centroid.latitude.is_nan());
        assert!(!centroid.longitude.is_nan());
    }

    #[test]
    fn test_cell_area_reasonable() {
        let cs = test_coord_system();
        let mut field = make_empty_field(100);

        let gate_start = 40;
        let gate_end = 45;
        let az_start = 20;
        let az_end = 25;
        let n_gates = (gate_end - gate_start) * (az_end - az_start);

        for az in az_start..az_end {
            for gate in gate_start..gate_end {
                field.set(az, gate, 45.0, GateStatus::Valid);
            }
        }

        let detector = StormCellDetector::new(35.0, 5).unwrap();
        let cells = detector.detect(&field, &cs).unwrap();

        assert_eq!(cells.len(), 1);
        assert_eq!(cells[0].gate_count(), n_gates);

        // Expected area: sum of annular sector areas
        let az_spacing_rad = (1.0f64).to_radians();
        let gate_interval = 0.25;
        let first_gate = 2.0;
        let mut expected_area = 0.0;
        for gate in gate_start..gate_end {
            let range_km = first_gate + gate as f64 * gate_interval;
            expected_area += az_spacing_rad * range_km * gate_interval * (az_end - az_start) as f64;
        }

        let actual = cells[0].area_km2();
        let ratio = actual / expected_area;
        assert!(
            (0.9..=1.1).contains(&ratio),
            "area {} not within 10% of expected {}",
            actual,
            expected_area
        );
    }

    #[test]
    fn test_nodata_gates_not_included() {
        let cs = test_coord_system();
        let mut field = make_empty_field(100);

        // Create a 3x3 block with a NoData hole in the center
        for az in 10..13 {
            for gate in 20..23 {
                field.set(az, gate, 40.0, GateStatus::Valid);
            }
        }
        // Punch a hole — this gate stays NoData so it's not part of any cell
        field.set(11, 21, 0.0, GateStatus::NoData);

        let detector = StormCellDetector::new(35.0, 1).unwrap();
        let cells = detector.detect(&field, &cs).unwrap();

        // The NoData gate should not be counted
        let total_gates: usize = cells.iter().map(|c| c.gate_count()).sum();
        assert_eq!(total_gates, 8); // 9 - 1 NoData
    }

    #[test]
    fn test_varying_reflectivity_weighted_centroid() {
        let cs = test_coord_system();
        let mut field = make_empty_field(100);

        // Create a cell where one end is much stronger
        // Low end: gates 40-42 at 36 dBZ
        for gate in 40..43 {
            field.set(90, gate, 36.0, GateStatus::Valid);
        }
        // High end: gates 43-45 at 60 dBZ
        for gate in 43..46 {
            field.set(90, gate, 60.0, GateStatus::Valid);
        }

        let detector = StormCellDetector::new(35.0, 3).unwrap();
        let cells = detector.detect(&field, &cs).unwrap();

        assert_eq!(cells.len(), 1);

        // Compute the unweighted geometric midpoint range
        let first_gate = field.first_gate_range_km();
        let interval = field.gate_interval_km();
        let mid_range = first_gate + 42.5 * interval; // middle gate of 40-45

        // The weighted centroid should be shifted toward the high-end (gate 43-45)
        // meaning further from the radar than the geometric midpoint
        let centroid_range_km = {
            let polar = cs.geo_to_polar(cells[0].centroid(), cells[0].elevation_degrees());
            polar.range_km
        };

        assert!(
            centroid_range_km > mid_range,
            "centroid range {:.3} should be beyond geometric mid {:.3} (shifted toward high-dBZ end)",
            centroid_range_km,
            mid_range
        );
    }
}
