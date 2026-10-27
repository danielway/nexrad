use std::collections::VecDeque;

use crate::azimuth::{covers_full_circle, neighbor_azimuth};
use crate::result::{check_finite, check_product, Error, Result};
use nexrad_model::data::{GateStatus, Product, SweepField};
use nexrad_model::geo::{GeoExtent, GeoPoint, PolarPoint, RadarCoordinateSystem};

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
    /// Geographic centroid of the cell, computed as the mean of all gate positions
    /// weighted by linear reflectivity factor.
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
    bounds: GeoExtent,
    /// Elevation angle of the sweep this cell was detected on (degrees).
    elevation_angle_degrees: f32,
    /// Azimuth of the gate with maximum reflectivity (degrees).
    max_reflectivity_azimuth_angle_degrees: f32,
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

    /// Geographic centroid of the cell.
    ///
    /// Gate positions are weighted by linear reflectivity factor, `10^(dBZ / 10)`, so the
    /// centroid leans toward the most intense echoes and every gate has a positive weight
    /// whatever the detection threshold.
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

    /// Geographic bounding box of the cell's gate centers.
    pub fn bounds(&self) -> GeoExtent {
        self.bounds
    }

    /// Elevation angle of the source sweep in degrees.
    pub fn elevation_angle_degrees(&self) -> f32 {
        self.elevation_angle_degrees
    }

    /// Azimuth of the maximum reflectivity gate (degrees).
    pub fn max_reflectivity_azimuth_angle_degrees(&self) -> f32 {
        self.max_reflectivity_azimuth_angle_degrees
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
///    (4-connectivity). When the field covers the full circle (the gap across
///    north is at most twice the azimuth spacing), the last azimuth index is
///    adjacent to the first; a sector's edges are not adjacent.
/// 3. Filter out components with fewer than `min_gate_count` gates.
/// 4. For each remaining component, compute cell properties.
///
/// # Example
///
/// ```
/// use nexrad_process::detection::StormCellDetector;
/// # let site = nexrad_model::meta::Site::builder(*b"KTLX", 35.3331, -97.2778).build()?;
/// # let coord_system = nexrad_model::geo::RadarCoordinateSystem::new(&site);
/// # let mut reflectivity_field = nexrad_model::data::SweepField::builder("Reflectivity", "dBZ")
/// #     .product(nexrad_model::data::Product::Reflectivity)
/// #     .elevation_angle_degrees(0.5)
/// #     .azimuth_angles_degrees((0..360).map(|a| a as f32).collect())
/// #     .first_gate_range_km(2.0)
/// #     .gate_interval_km(0.25)
/// #     .gate_count(400)
/// #     .build()?;
/// # for azimuth in 40..50 {
/// #     for gate in 100..120 {
/// #         reflectivity_field.set(azimuth, gate, 50.0, nexrad_model::data::GateStatus::Valid)?;
/// #     }
/// # }
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
/// # assert_eq!(cells.len(), 1);
/// # Ok::<(), nexrad_process::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq)]
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
    /// Returns [`Error::InvalidParameter`] if `reflectivity_threshold_dbz` is not finite or
    /// `min_gate_count` is zero.
    pub fn new(reflectivity_threshold_dbz: f32, min_gate_count: usize) -> Result<Self> {
        check_finite("reflectivity_threshold_dbz", reflectivity_threshold_dbz)?;
        if min_gate_count == 0 {
            return Err(Error::InvalidParameter {
                parameter: "min_gate_count",
                reason: "must be greater than zero",
            });
        }
        Ok(Self {
            reflectivity_threshold_dbz,
            min_gate_count,
        })
    }

    /// Minimum reflectivity in dBZ for a gate to be part of a cell.
    pub fn reflectivity_threshold_dbz(&self) -> f32 {
        self.reflectivity_threshold_dbz
    }

    /// Minimum number of contiguous gates for a region to be reported as a cell.
    pub fn min_gate_count(&self) -> usize {
        self.min_gate_count
    }

    /// Detect storm cells in the given reflectivity sweep field.
    ///
    /// # Parameters
    ///
    /// - `field` — A sweep field holding [`Product::Reflectivity`] (dBZ values).
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
    /// Returns [`Error::UnexpectedProduct`] if the field does not hold
    /// [`Product::Reflectivity`] and [`Error::InvalidGeometry`] if it has zero
    /// azimuths or zero gates.
    pub fn detect(
        &self,
        field: &SweepField,
        coord_system: &RadarCoordinateSystem,
    ) -> Result<Vec<StormCell>> {
        check_product(field, Product::Reflectivity)?;

        let az_count = field.azimuth_count();
        let num_gates = field.gate_count();
        let wraps = covers_full_circle(field);

        if az_count == 0 {
            return Err(Error::InvalidGeometry {
                parameter: "azimuth_count",
                reason: "must be greater than zero",
            });
        }
        if num_gates == 0 {
            return Err(Error::InvalidGeometry {
                parameter: "gate_count",
                reason: "must be greater than zero",
            });
        }

        let total = az_count * num_gates;

        // Phase 1: Build threshold mask
        let above_threshold: Vec<bool> = field
            .values()
            .iter()
            .zip(field.statuses())
            .map(|(&val, &status)| {
                status == GateStatus::Valid && val >= self.reflectivity_threshold_dbz
            })
            .collect();

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

                    // 4-connected neighbors: azimuth ± 1 (wrapping only across a full
                    // circle), range ± 1 (clamped)
                    for offset in [-1, 1] {
                        let Some(n_az) = neighbor_azimuth(az, offset, az_count, wraps) else {
                            continue;
                        };
                        let n_idx = n_az * num_gates + gate;
                        if above_threshold[n_idx] && !visited[n_idx] {
                            visited[n_idx] = true;
                            queue.push_back((n_az, gate));
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
            let Some((val, _)) = field.get(az_idx, gate_idx) else {
                continue;
            };

            let azimuth_deg = field.azimuth_angles_degrees()[az_idx];
            let range_km = field.first_gate_range_km() + gate_idx as f64 * field.gate_interval_km();

            // Reflectivity statistics
            sum_dbz += val as f64;
            if val > max_dbz {
                max_dbz = val;
                max_dbz_az = azimuth_deg;
                max_dbz_range = range_km;
            }

            // Geographic position of this gate
            let geo = coord_system.polar_to_geo(PolarPoint {
                azimuth_angle_degrees: azimuth_deg as f64,
                range_km,
                elevation_angle_degrees: field.elevation_angle_degrees() as f64,
            });

            // Centroid weighted by linear reflectivity factor (mm^6/m^3), which is positive
            // for any dBZ value.
            let weight = 10f64.powf(f64::from(val) / 10.0);
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
            bounds: GeoExtent {
                min: GeoPoint {
                    latitude: min_lat,
                    longitude: min_lon,
                },
                max: GeoPoint {
                    latitude: max_lat,
                    longitude: max_lon,
                },
            },
            elevation_angle_degrees: field.elevation_angle_degrees(),
            max_reflectivity_azimuth_angle_degrees: max_dbz_az,
            max_reflectivity_range_km: max_dbz_range,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexrad_model::meta::Site;

    fn test_site() -> Site {
        Site::builder(*b"KTLX", 35.3331, -97.2778)
            .site_elevation_meters(370)
            .feedhorn_height_meters(10)
            .build()
            .unwrap()
    }

    fn test_coord_system() -> RadarCoordinateSystem {
        RadarCoordinateSystem::new(&test_site())
    }

    /// Create a 360-azimuth, `gate_count`-gate field with all gates set to NoData.
    fn make_empty_field(gate_count: usize) -> SweepField {
        let azimuths: Vec<f32> = (0..360).map(|i| i as f32).collect();
        SweepField::builder("Reflectivity", "dBZ")
            .product(Product::Reflectivity)
            .elevation_angle_degrees(0.5)
            .azimuth_angles_degrees(azimuths)
            .first_gate_range_km(2.0)
            .gate_interval_km(0.25)
            .gate_count(gate_count)
            .build()
            .expect("valid geometry")
    }

    /// Create a uniform field where all gates have the given value.
    fn make_uniform_field(gate_count: usize, value: f32) -> SweepField {
        let mut field = make_empty_field(gate_count);
        for az in 0..360 {
            for gate in 0..gate_count {
                field.set(az, gate, value, GateStatus::Valid).unwrap();
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
                field.set(az, gate, 40.0, GateStatus::Valid).unwrap();
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
                field.set(az, gate, 50.0, GateStatus::Valid).unwrap();
            }
        }

        // Block B: weaker, well-separated
        for az in 100..105 {
            for gate in 50..55 {
                field.set(az, gate, 40.0, GateStatus::Valid).unwrap();
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
        field.set(50, 30, 60.0, GateStatus::Valid).unwrap();

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
            field.set(az, 30, 40.0, GateStatus::Valid).unwrap();
        }

        let detector = StormCellDetector::new(35.0, 3).unwrap();
        let cells = detector.detect(&field, &cs).unwrap();

        // Should be a single cell, not two
        assert_eq!(cells.len(), 1);
        assert_eq!(cells[0].gate_count(), 5);
    }

    #[test]
    fn test_sector_edges_not_joined() -> Result<()> {
        let cs = test_coord_system();
        let mut field = SweepField::builder("Reflectivity", "dBZ")
            .product(Product::Reflectivity)
            .elevation_angle_degrees(0.5)
            .azimuth_angles_degrees((0..90).map(|az| az as f32).collect())
            .first_gate_range_km(2.0)
            .gate_interval_km(0.25)
            .gate_count(100)
            .build()?;

        // Gates on both edges of a 90-degree sector: radials 88, 89 and 0, 1.
        for az in [88, 89, 0, 1] {
            field.set(az, 30, 40.0, GateStatus::Valid)?;
        }

        let cells = StormCellDetector::new(35.0, 1)?.detect(&field, &cs)?;
        assert_eq!(cells.len(), 2);
        for cell in &cells {
            assert_eq!(cell.gate_count(), 2);
        }
        Ok(())
    }

    #[test]
    fn test_detect_requires_reflectivity() {
        let cs = test_coord_system();
        let velocity = SweepField::builder("Velocity", "m/s")
            .product(Product::Velocity)
            .azimuth_angles_degrees(vec![0.0, 1.0])
            .gate_interval_km(0.25)
            .gate_count(4)
            .build()
            .unwrap();
        let detector = StormCellDetector::new(35.0, 1).unwrap();
        assert_eq!(
            detector.detect(&velocity, &cs),
            Err(Error::UnexpectedProduct {
                expected: Product::Reflectivity,
                actual: Some(Product::Velocity),
            })
        );
    }

    #[test]
    fn test_invalid_min_gate_count_zero() {
        let result = StormCellDetector::new(35.0, 0);
        assert!(matches!(
            result,
            Err(Error::InvalidParameter {
                parameter: "min_gate_count",
                ..
            })
        ));
    }

    #[test]
    fn test_invalid_threshold_not_finite() {
        for threshold in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(matches!(
                StormCellDetector::new(threshold, 5),
                Err(Error::InvalidParameter {
                    parameter: "reflectivity_threshold_dbz",
                    ..
                })
            ));
        }
    }

    #[test]
    fn test_detector_getters() -> Result<()> {
        let detector = StormCellDetector::new(-5.0, 7)?;
        assert_eq!(detector.reflectivity_threshold_dbz(), -5.0);
        assert_eq!(detector.min_gate_count(), 7);
        Ok(())
    }

    #[test]
    fn test_bounds_contain_centroid() -> Result<()> {
        let cs = test_coord_system();
        let mut field = make_empty_field(100);
        for az in 10..15 {
            for gate in 20..25 {
                field.set(az, gate, 40.0, GateStatus::Valid)?;
            }
        }

        let cells = StormCellDetector::new(35.0, 5)?.detect(&field, &cs)?;
        let bounds = cells[0].bounds();
        let centroid = cells[0].centroid();
        assert!(bounds.min.latitude < bounds.max.latitude);
        assert!(bounds.min.longitude < bounds.max.longitude);
        assert!((bounds.min.latitude..=bounds.max.latitude).contains(&centroid.latitude));
        assert!((bounds.min.longitude..=bounds.max.longitude).contains(&centroid.longitude));
        Ok(())
    }

    #[test]
    fn test_centroid_with_zero_dbz_threshold() -> Result<()> {
        let cs = test_coord_system();
        let mut field = make_empty_field(100);

        // A radial run at exactly 0 dBZ: raw-dBZ weights would all be zero.
        for gate in 40..46 {
            field.set(90, gate, 0.0, GateStatus::Valid)?;
        }

        let cells = StormCellDetector::new(0.0, 3)?.detect(&field, &cs)?;
        assert_eq!(cells.len(), 1);
        let centroid = cells[0].centroid();
        assert!(centroid.latitude.is_finite() && centroid.longitude.is_finite());

        // Equal weights put the centroid at the geometric middle of the run.
        let polar = cs
            .geo_to_polar(centroid, cells[0].elevation_angle_degrees() as f64)
            .unwrap();
        let mid_range = field.first_gate_range_km() + 42.5 * field.gate_interval_km();
        assert!(
            (polar.range_km - mid_range).abs() < 0.01,
            "centroid range {:.3} should be at the run's middle {:.3}",
            polar.range_km,
            mid_range
        );
        Ok(())
    }

    #[test]
    fn test_centroid_with_negative_dbz_leans_to_stronger_echo() -> Result<()> {
        let cs = test_coord_system();
        let mut field = make_empty_field(100);

        // Weak near end at -10 dBZ, stronger far end at 0 dBZ. Raw-dBZ weights would give the
        // 0 dBZ gates no weight and put the centroid on the weaker end.
        for gate in 40..43 {
            field.set(90, gate, -10.0, GateStatus::Valid)?;
        }
        for gate in 43..46 {
            field.set(90, gate, 0.0, GateStatus::Valid)?;
        }

        let cells = StormCellDetector::new(-15.0, 3)?.detect(&field, &cs)?;
        assert_eq!(cells.len(), 1);
        let polar = cs
            .geo_to_polar(
                cells[0].centroid(),
                cells[0].elevation_angle_degrees() as f64,
            )
            .unwrap();
        let first = field.first_gate_range_km();
        let interval = field.gate_interval_km();
        let mid_range = first + 42.5 * interval;
        assert!(
            polar.range_km > mid_range,
            "centroid should lean to the 0 dBZ end"
        );
        assert!(polar.range_km <= first + 45.0 * interval + 0.01);
        Ok(())
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
                field.set(az, gate, 45.0, GateStatus::Valid).unwrap();
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
                field.set(az, gate, 45.0, GateStatus::Valid).unwrap();
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
                field.set(az, gate, 40.0, GateStatus::Valid).unwrap();
            }
        }
        // Punch a hole — this gate stays NoData so it's not part of any cell
        field.set(11, 21, 0.0, GateStatus::NoData).unwrap();

        let detector = StormCellDetector::new(35.0, 1).unwrap();
        let cells = detector.detect(&field, &cs).unwrap();

        // The NoData gate should not be counted
        let total_gates: usize = cells.iter().map(|c| c.gate_count()).sum();
        assert_eq!(total_gates, 8); // 9 - 1 NoData
    }

    #[test]
    fn test_filtered_gates_not_included() {
        let cs = test_coord_system();
        let mut field = make_empty_field(100);

        // A radial run of five gates whose middle gate was filtered with a strong value
        for gate in 20..25 {
            field.set(10, gate, 40.0, GateStatus::Valid).unwrap();
        }
        field.set(10, 22, 60.0, GateStatus::Filtered).unwrap();

        let detector = StormCellDetector::new(35.0, 1).unwrap();
        let cells = detector.detect(&field, &cs).unwrap();

        // The filtered gate splits the run and contributes neither area nor intensity
        assert_eq!(cells.len(), 2);
        for cell in &cells {
            assert_eq!(cell.gate_count(), 2);
            assert_eq!(cell.max_reflectivity_dbz(), 40.0);
        }
    }

    #[test]
    fn test_varying_reflectivity_weighted_centroid() {
        let cs = test_coord_system();
        let mut field = make_empty_field(100);

        // Create a cell where one end is much stronger
        // Low end: gates 40-42 at 36 dBZ
        for gate in 40..43 {
            field.set(90, gate, 36.0, GateStatus::Valid).unwrap();
        }
        // High end: gates 43-45 at 60 dBZ
        for gate in 43..46 {
            field.set(90, gate, 60.0, GateStatus::Valid).unwrap();
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
            let polar = cs
                .geo_to_polar(
                    cells[0].centroid(),
                    cells[0].elevation_angle_degrees() as f64,
                )
                .unwrap();
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
