use super::sweep::median;
use crate::data::{GateStatus, MomentData, MomentValue, Product, Radial};
use crate::geo::{GeoExtent, GeoPoint};
use crate::result::{Error, Result};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A 2D polar field of floating-point values for one sweep of one product.
///
/// This is the universal data surface consumed by both the renderer and processing
/// algorithms. It can be constructed from raw decoded radials or produced as the
/// output of a processing step.
///
/// Values are stored as parallel `f32` and [`GateStatus`] arrays in row-major order:
/// `values[azimuth_index * gate_count + gate_index]`. This layout provides contiguous
/// float slices for efficient numerical processing.
///
/// # Construction
///
/// ```
/// use nexrad_model::data::{SweepField, Product};
/// # use nexrad_model::data::{MomentData, Radial, Sweep};
/// # let reflectivity = MomentData::builder(Product::Reflectivity, vec![100, 110, 120])
/// #     .first_gate_range_meters(2000)
/// #     .gate_interval_meters(250)
/// #     .scale(2.0)
/// #     .offset(66.0)
/// #     .build()?;
/// # let radial = Radial::builder(0, 0.0, 1, 0.5).moment(reflectivity).build()?;
/// # let sweep = Sweep::new(1, vec![radial])?;
///
/// let field = SweepField::from_radials(sweep.radials(), Product::Reflectivity)
///     .expect("reflectivity data present");
/// # assert_eq!(field.gate_count(), 3);
/// # Ok::<(), nexrad_model::result::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct SweepField {
    label: String,
    unit: String,
    product: Option<Product>,
    elevation_angle_degrees: f32,
    azimuth_angles_degrees: Vec<f32>,
    azimuth_spacing_degrees: f32,
    first_gate_range_km: f64,
    gate_interval_km: f64,
    gate_count: usize,
    values: Vec<f32>,
    statuses: Vec<GateStatus>,
}

/// The geometry and row order of a sweep field extracted from radials.
struct RadialLayout {
    first_gate_range_km: f64,
    gate_interval_km: f64,
    gate_count: usize,
    elevation_angle_degrees: f32,
    azimuth_spacing_degrees: f32,
    /// `(azimuth, index into the radials)` for each row, sorted by azimuth.
    rows: Vec<(f32, usize)>,
}

impl SweepField {
    /// Create a `SweepField` from decoded radials for a specific product.
    ///
    /// Extracts and decodes moment data from each radial, converting [`MomentValue`]
    /// into `(f32, GateStatus)` pairs. Radials are sorted by azimuth angle.
    ///
    /// The gate geometry (first gate range, gate interval, and gate count) is taken from the
    /// first radial carrying the product. Radials whose first gate range or gate interval differ
    /// from it, or whose moment data has no gates, are skipped as if they did not carry the
    /// product. Radials with fewer gates are padded with [`GateStatus::NoData`]; radials with
    /// more gates are truncated to the gate count.
    ///
    /// The elevation angle is the median of the included radials' elevation angles and the
    /// azimuth spacing is that of the first included radial.
    ///
    /// Returns `None` if no radials contain data for the requested product.
    pub fn from_radials(radials: &[Radial], product: Product) -> Option<Self> {
        let layout = Self::layout(radials, product)?;
        let gate_count = layout.gate_count;

        let total_gates = layout.rows.len() * gate_count;
        let mut values = vec![0.0f32; total_gates];
        let mut status = vec![GateStatus::NoData; total_gates];

        // Decode directly into the final flat arrays to avoid per-radial buffers.
        for ((row_values, row_status), &(_, radial_idx)) in values
            .chunks_exact_mut(gate_count)
            .zip(status.chunks_exact_mut(gate_count))
            .zip(&layout.rows)
        {
            if let Some(moment) = radials.get(radial_idx).and_then(|r| r.moment(product)) {
                Self::decode_moment_into(moment, row_values, row_status);
            }
        }

        Some(Self::from_layout(layout, product, values, status))
    }

    /// Create a `SweepField` from decoded radials, consuming the radials.
    ///
    /// Same as [`from_radials`](Self::from_radials) but takes ownership of the radials,
    /// allowing them to be dropped immediately after extraction. Each radial's encoded
    /// moment data is freed as soon as it has been decoded into the field, reducing
    /// peak memory usage when you no longer need the original radials.
    pub fn from_radials_owned(mut radials: Vec<Radial>, product: Product) -> Option<Self> {
        let layout = Self::layout(&radials, product)?;
        let gate_count = layout.gate_count;

        let total_gates = layout.rows.len() * gate_count;
        let mut values = vec![0.0f32; total_gates];
        let mut status = vec![GateStatus::NoData; total_gates];

        // Process in descending original-index order so that swap_remove doesn't invalidate
        // indices we haven't visited yet.
        let mut work: Vec<(usize, usize)> = layout
            .rows
            .iter()
            .enumerate()
            .map(|(row, &(_, orig_idx))| (row, orig_idx))
            .collect();
        work.sort_by_key(|&(_, orig_idx)| std::cmp::Reverse(orig_idx));

        for (row, orig_idx) in work {
            let radial = radials.swap_remove(orig_idx);
            let row_range = row * gate_count..(row + 1) * gate_count;
            if let (Some(moment), Some(row_values), Some(row_status)) = (
                radial.moment(product),
                values.get_mut(row_range.clone()),
                status.get_mut(row_range),
            ) {
                Self::decode_moment_into(moment, row_values, row_status);
            }
            // `radial` is dropped here, freeing its encoded moment bytes
        }

        Some(Self::from_layout(layout, product, values, status))
    }

    /// Start building an empty field (for processing output) with the given label and unit.
    ///
    /// The geometry is set with the builder's setters; see [`SweepFieldBuilder`] for defaults
    /// and validation rules.
    pub fn builder(label: impl Into<String>, unit: impl Into<String>) -> SweepFieldBuilder {
        SweepFieldBuilder {
            label: label.into(),
            unit: unit.into(),
            product: None,
            elevation_angle_degrees: 0.0,
            azimuth_angles_degrees: Vec::new(),
            azimuth_spacing_degrees: 1.0,
            first_gate_range_km: 0.0,
            gate_interval_km: 0.0,
            gate_count: 0,
        }
    }

    /// Create a new field with the same geometry and [`product`](Self::product) as this one
    /// but empty data.
    ///
    /// All values are initialized to `0.0` with [`GateStatus::NoData`].
    #[must_use]
    pub fn new_like(&self, label: impl Into<String>, unit: impl Into<String>) -> Self {
        let total = self.values.len();
        Self {
            label: label.into(),
            unit: unit.into(),
            product: self.product,
            elevation_angle_degrees: self.elevation_angle_degrees,
            azimuth_angles_degrees: self.azimuth_angles_degrees.clone(),
            azimuth_spacing_degrees: self.azimuth_spacing_degrees,
            first_gate_range_km: self.first_gate_range_km,
            gate_interval_km: self.gate_interval_km,
            gate_count: self.gate_count,
            values: vec![0.0; total],
            statuses: vec![GateStatus::NoData; total],
        }
    }

    // --- Accessors ---

    /// Human-readable label for this field.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Physical unit string.
    pub fn unit(&self) -> &str {
        &self.unit
    }

    /// The radar product this field holds, if any.
    ///
    /// Set by [`from_radials`](Self::from_radials) and
    /// [`from_radials_owned`](Self::from_radials_owned), copied by [`new_like`](Self::new_like),
    /// and optionally set on fields built with [`builder`](Self::builder).
    pub fn product(&self) -> Option<Product> {
        self.product
    }

    /// Elevation angle in degrees.
    pub fn elevation_angle_degrees(&self) -> f32 {
        self.elevation_angle_degrees
    }

    /// Number of azimuth rows.
    pub fn azimuth_count(&self) -> usize {
        self.azimuth_angles_degrees.len()
    }

    /// Number of gates per radial.
    pub fn gate_count(&self) -> usize {
        self.gate_count
    }

    /// Azimuth angles for each radial row (sorted ascending).
    pub fn azimuth_angles_degrees(&self) -> &[f32] {
        &self.azimuth_angles_degrees
    }

    /// Azimuthal spacing between radials in degrees.
    pub fn azimuth_spacing_degrees(&self) -> f32 {
        self.azimuth_spacing_degrees
    }

    /// Range to center of first gate in km.
    pub fn first_gate_range_km(&self) -> f64 {
        self.first_gate_range_km
    }

    /// Gate spacing in km.
    pub fn gate_interval_km(&self) -> f64 {
        self.gate_interval_km
    }

    /// Maximum range of the data in km.
    pub fn max_range_km(&self) -> f64 {
        self.first_gate_range_km + (self.gate_count as f64) * self.gate_interval_km
    }

    /// All gate values as a flat slice (row-major: `values[az_idx * gate_count + gate_idx]`).
    pub fn values(&self) -> &[f32] {
        &self.values
    }

    /// All gate statuses as a flat slice.
    pub fn statuses(&self) -> &[GateStatus] {
        &self.statuses
    }

    // --- Per-gate access ---

    /// Get the value and status at a specific (azimuth_index, gate_index).
    ///
    /// Returns `None` if `azimuth_idx` is not less than [`azimuth_count`](Self::azimuth_count)
    /// or `gate_idx` is not less than [`gate_count`](Self::gate_count).
    pub fn get(&self, azimuth_idx: usize, gate_idx: usize) -> Option<(f32, GateStatus)> {
        let idx = flat_index(azimuth_idx, gate_idx, self.azimuth_count(), self.gate_count)?;
        Some((*self.values.get(idx)?, *self.statuses.get(idx)?))
    }

    /// Set the value and status at a specific (azimuth_index, gate_index).
    ///
    /// # Errors
    ///
    /// Returns [`Error::IndexOutOfBounds`] if either index is outside the field's dimensions.
    pub fn set(
        &mut self,
        azimuth_idx: usize,
        gate_idx: usize,
        value: f32,
        status: GateStatus,
    ) -> Result<()> {
        let rows = self.azimuth_count();
        let columns = self.gate_count;
        set_cell(
            &mut self.values,
            &mut self.statuses,
            (azimuth_idx, gate_idx),
            (rows, columns),
            (value, status),
        )
    }

    /// Get a mutable slice of values for a full radial row.
    ///
    /// Returns `None` if `azimuth_idx` is not less than [`azimuth_count`](Self::azimuth_count).
    pub fn radial_values_mut(&mut self, azimuth_idx: usize) -> Option<&mut [f32]> {
        let range = self.row_range(azimuth_idx)?;
        self.values.get_mut(range)
    }

    /// Get a mutable slice of statuses for a full radial row.
    ///
    /// Returns `None` if `azimuth_idx` is not less than [`azimuth_count`](Self::azimuth_count).
    pub fn radial_statuses_mut(&mut self, azimuth_idx: usize) -> Option<&mut [GateStatus]> {
        let range = self.row_range(azimuth_idx)?;
        self.statuses.get_mut(range)
    }

    /// Get an immutable slice of values for a full radial row.
    ///
    /// Returns `None` if `azimuth_idx` is not less than [`azimuth_count`](Self::azimuth_count).
    pub fn radial_values(&self, azimuth_idx: usize) -> Option<&[f32]> {
        self.values.get(self.row_range(azimuth_idx)?)
    }

    /// Get an immutable slice of statuses for a full radial row.
    ///
    /// Returns `None` if `azimuth_idx` is not less than [`azimuth_count`](Self::azimuth_count).
    pub fn radial_statuses(&self, azimuth_idx: usize) -> Option<&[GateStatus]> {
        self.statuses.get(self.row_range(azimuth_idx)?)
    }

    // --- Queries ---

    /// Query the value at a polar coordinate (finds nearest azimuth and gate).
    ///
    /// The azimuth is normalized into `0.0..360.0` first. When the field's azimuths cover the
    /// full circle, the nearest azimuth is matched across the 0/360 boundary. Otherwise (a
    /// sector or partial sweep), azimuths more than half the azimuth spacing outside the covered
    /// sector do not match.
    ///
    /// Returns `None` if the coordinate is outside the field's range or covered azimuths.
    pub fn value_at_polar(&self, azimuth_degrees: f64, range_km: f64) -> Option<(f32, GateStatus)> {
        if !(range_km >= self.first_gate_range_km && range_km < self.max_range_km()) {
            return None;
        }

        let az_idx = self.find_nearest_azimuth(azimuth_degrees)?;
        let gate_idx = ((range_km - self.first_gate_range_km) / self.gate_interval_km) as usize;

        self.get(az_idx, gate_idx)
    }

    /// Compute the (min, max) of valid values in this field.
    ///
    /// Returns `None` if the field contains no valid values.
    pub fn value_range(&self) -> Option<(f32, f32)> {
        valid_value_range(&self.values, &self.statuses)
    }

    // --- Internal helpers ---

    /// The flat index range of an azimuth row, if the row exists.
    fn row_range(&self, azimuth_idx: usize) -> Option<std::ops::Range<usize>> {
        if azimuth_idx >= self.azimuth_count() {
            return None;
        }
        let start = azimuth_idx * self.gate_count;
        Some(start..start + self.gate_count)
    }

    /// Find the nearest azimuth index for the given angle.
    ///
    /// Inside the span of the field's azimuths the nearest row always matches. Outside it, the
    /// first or last row matches across the 0/360 boundary when the azimuths cover the full
    /// circle, and only within half the azimuth spacing otherwise.
    fn find_nearest_azimuth(&self, azimuth_degrees: f64) -> Option<usize> {
        if !azimuth_degrees.is_finite() {
            return None;
        }
        let mut azimuth = azimuth_degrees.rem_euclid(360.0);
        // rem_euclid can round a tiny negative input up to exactly 360.0.
        if azimuth >= 360.0 {
            azimuth = 0.0;
        }

        let angles = &self.azimuth_angles_degrees;
        let first = f64::from(*angles.first()?);
        let last = f64::from(*angles.last()?);
        let last_idx = angles.len() - 1;

        if (first..=last).contains(&azimuth) {
            let pos = angles.partition_point(|&a| f64::from(a) < azimuth);
            let Some(&next) = angles.get(pos) else {
                return Some(last_idx);
            };
            let Some(prev_idx) = pos.checked_sub(1) else {
                return Some(0);
            };
            let prev = angles.get(prev_idx).map_or(f64::MIN, |&a| f64::from(a));
            return if azimuth - prev <= f64::from(next) - azimuth {
                Some(prev_idx)
            } else {
                Some(pos)
            };
        }

        // Outside the azimuth span: distances through the gap that wraps past 0/360.
        let distance_to_first = (first - azimuth).rem_euclid(360.0);
        let distance_to_last = (azimuth - last).rem_euclid(360.0);
        let (nearest_idx, distance) = if distance_to_first < distance_to_last {
            (0, distance_to_first)
        } else {
            (last_idx, distance_to_last)
        };

        let spacing = f64::from(self.azimuth_spacing_degrees);
        let wrap_gap = 360.0 - (last - first);
        let full_circle = wrap_gap <= 2.0 * spacing;
        if full_circle || distance <= spacing / 2.0 {
            Some(nearest_idx)
        } else {
            None
        }
    }

    /// Determine the geometry and row order of a field for `product` from `radials`.
    fn layout(radials: &[Radial], product: Product) -> Option<RadialLayout> {
        let usable =
            |moment: &MomentData| moment.gate_count() > 0 && moment.gate_interval_meters() > 0;
        let reference = radials
            .iter()
            .filter_map(|radial| radial.moment(product))
            .find(|moment| usable(moment))?;
        let first_gate_range_meters = reference.first_gate_range_meters();
        let gate_interval_meters = reference.gate_interval_meters();

        let included: Vec<(usize, &Radial)> = radials
            .iter()
            .enumerate()
            .filter(|(_, radial)| {
                radial.moment(product).is_some_and(|moment| {
                    usable(moment)
                        && moment.first_gate_range_meters() == first_gate_range_meters
                        && moment.gate_interval_meters() == gate_interval_meters
                })
            })
            .collect();

        let azimuth_spacing_degrees = included.first()?.1.azimuth_spacing_degrees();
        let mut elevations: Vec<f32> = included
            .iter()
            .map(|(_, radial)| radial.elevation_angle_degrees())
            .collect();
        let elevation_angle_degrees = median(&mut elevations)?;

        let mut rows: Vec<(f32, usize)> = included
            .iter()
            .map(|&(idx, radial)| (radial.azimuth_angle_degrees(), idx))
            .collect();
        rows.sort_by(|a, b| a.0.total_cmp(&b.0));

        Some(RadialLayout {
            first_gate_range_km: reference.first_gate_range_km(),
            gate_interval_km: reference.gate_interval_km(),
            gate_count: reference.gate_count(),
            elevation_angle_degrees,
            azimuth_spacing_degrees,
            rows,
        })
    }

    /// Assemble a field from a layout and its decoded values.
    fn from_layout(
        layout: RadialLayout,
        product: Product,
        values: Vec<f32>,
        statuses: Vec<GateStatus>,
    ) -> Self {
        Self {
            label: product.label().to_string(),
            unit: product.unit().to_string(),
            product: Some(product),
            elevation_angle_degrees: layout.elevation_angle_degrees,
            azimuth_angles_degrees: layout.rows.iter().map(|&(az, _)| az).collect(),
            azimuth_spacing_degrees: layout.azimuth_spacing_degrees,
            first_gate_range_km: layout.first_gate_range_km,
            gate_interval_km: layout.gate_interval_km,
            gate_count: layout.gate_count,
            values,
            statuses,
        }
    }

    /// Decode moment data directly into pre-allocated row slices.
    ///
    /// Gates beyond what the moment provides are left at their initial values (0.0 / NoData),
    /// and gates beyond the row length are ignored.
    fn decode_moment_into(moment: &MomentData, values: &mut [f32], status: &mut [GateStatus]) {
        for ((value, gate_status), mv) in
            values.iter_mut().zip(status.iter_mut()).zip(moment.iter())
        {
            match mv {
                MomentValue::Value(v) => {
                    *value = v;
                    *gate_status = GateStatus::Valid;
                }
                MomentValue::BelowThreshold => {
                    *gate_status = GateStatus::BelowThreshold;
                }
                MomentValue::RangeFolded => {
                    *gate_status = GateStatus::RangeFolded;
                }
                MomentValue::CfpStatus(_) => {
                    // value stays 0.0, status stays NoData (initial values)
                }
            }
        }
    }
}

/// Builder for an empty [`SweepField`], created with [`SweepField::builder`].
///
/// The label and unit are required. The product and geometry default to:
///
/// - `product`: `None`
/// - `elevation_angle_degrees`: `0.0`
/// - `azimuth_angles_degrees`: empty
/// - `azimuth_spacing_degrees`: `1.0`
/// - `first_gate_range_km`: `0.0`
/// - `gate_interval_km`: `0.0`
/// - `gate_count`: `0`
///
/// [`build`](Self::build) returns [`Error::InvalidGeometry`] unless:
///
/// - `azimuth_angles_degrees` is non-empty, strictly ascending, and every angle is in
///   `0.0..360.0`
/// - `azimuth_spacing_degrees` and `gate_interval_km` are finite and greater than zero
/// - `first_gate_range_km` is finite and not negative
/// - `elevation_angle_degrees` is finite
/// - `gate_count` is greater than zero
///
/// All values of the built field are initialized to `0.0` with [`GateStatus::NoData`].
///
/// # Example
///
/// ```
/// use nexrad_model::data::SweepField;
///
/// let field = SweepField::builder("Reflectivity", "dBZ")
///     .elevation_angle_degrees(0.5)
///     .azimuth_angles_degrees(vec![0.0, 1.0, 2.0])
///     .first_gate_range_km(2.0)
///     .gate_interval_km(0.25)
///     .gate_count(100)
///     .build()?;
/// assert_eq!(field.azimuth_count(), 3);
/// assert_eq!(field.gate_count(), 100);
/// # Ok::<(), nexrad_model::result::Error>(())
/// ```
#[derive(Debug, Clone)]
#[must_use]
pub struct SweepFieldBuilder {
    label: String,
    unit: String,
    product: Option<Product>,
    elevation_angle_degrees: f32,
    azimuth_angles_degrees: Vec<f32>,
    azimuth_spacing_degrees: f32,
    first_gate_range_km: f64,
    gate_interval_km: f64,
    gate_count: usize,
}

impl SweepFieldBuilder {
    /// Set the radar product this field holds. Defaults to `None`.
    pub fn product(mut self, product: Product) -> Self {
        self.product = Some(product);
        self
    }

    /// Set the elevation angle in degrees. Defaults to `0.0`; must be finite.
    pub fn elevation_angle_degrees(mut self, elevation_angle_degrees: f32) -> Self {
        self.elevation_angle_degrees = elevation_angle_degrees;
        self
    }

    /// Set the azimuth angle in degrees for each radial row. Defaults to empty; must be
    /// non-empty, strictly ascending, and within `0.0..360.0` for [`build`](Self::build) to
    /// succeed.
    pub fn azimuth_angles_degrees(mut self, azimuth_angles_degrees: Vec<f32>) -> Self {
        self.azimuth_angles_degrees = azimuth_angles_degrees;
        self
    }

    /// Set the azimuthal spacing between radials in degrees. Defaults to `1.0`; must be finite
    /// and greater than zero.
    pub fn azimuth_spacing_degrees(mut self, azimuth_spacing_degrees: f32) -> Self {
        self.azimuth_spacing_degrees = azimuth_spacing_degrees;
        self
    }

    /// Set the range to the center of the first gate in km. Defaults to `0.0`; must be finite
    /// and not negative.
    pub fn first_gate_range_km(mut self, first_gate_range_km: f64) -> Self {
        self.first_gate_range_km = first_gate_range_km;
        self
    }

    /// Set the gate spacing in km. Defaults to `0.0`; must be finite and greater than zero for
    /// [`build`](Self::build) to succeed.
    pub fn gate_interval_km(mut self, gate_interval_km: f64) -> Self {
        self.gate_interval_km = gate_interval_km;
        self
    }

    /// Set the number of gates per radial. Defaults to `0`; must be greater than zero for
    /// [`build`](Self::build) to succeed.
    pub fn gate_count(mut self, gate_count: usize) -> Self {
        self.gate_count = gate_count;
        self
    }

    /// Build the empty [`SweepField`], with all values `0.0` and [`GateStatus::NoData`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidGeometry`] if any rule listed on [`SweepFieldBuilder`] is broken.
    pub fn build(self) -> Result<SweepField> {
        if self.azimuth_angles_degrees.is_empty() {
            return Err(invalid_geometry(
                "azimuth_angles_degrees",
                "must contain at least one azimuth",
            ));
        }
        if !self
            .azimuth_angles_degrees
            .iter()
            .all(|azimuth| (0.0..360.0).contains(azimuth))
        {
            return Err(invalid_geometry(
                "azimuth_angles_degrees",
                "must be finite and in 0.0..360.0",
            ));
        }
        if !self.azimuth_angles_degrees.windows(2).all(|w| w[0] < w[1]) {
            return Err(invalid_geometry(
                "azimuth_angles_degrees",
                "must be strictly ascending",
            ));
        }
        check_positive(
            "azimuth_spacing_degrees",
            f64::from(self.azimuth_spacing_degrees),
        )?;
        if !self.elevation_angle_degrees.is_finite() {
            return Err(invalid_geometry(
                "elevation_angle_degrees",
                "must be finite",
            ));
        }
        if !(self.first_gate_range_km.is_finite() && self.first_gate_range_km >= 0.0) {
            return Err(invalid_geometry(
                "first_gate_range_km",
                "must be finite and not negative",
            ));
        }
        check_positive("gate_interval_km", self.gate_interval_km)?;
        let total = check_dimensions(
            ("gate_count", self.gate_count),
            ("azimuth_angles_degrees", self.azimuth_angles_degrees.len()),
        )?;

        Ok(SweepField {
            label: self.label,
            unit: self.unit,
            product: self.product,
            elevation_angle_degrees: self.elevation_angle_degrees,
            azimuth_angles_degrees: self.azimuth_angles_degrees,
            azimuth_spacing_degrees: self.azimuth_spacing_degrees,
            first_gate_range_km: self.first_gate_range_km,
            gate_interval_km: self.gate_interval_km,
            gate_count: self.gate_count,
            values: vec![0.0; total],
            statuses: vec![GateStatus::NoData; total],
        })
    }
}

/// A 2D Cartesian grid of values on a geographic extent.
///
/// Used for products that combine data from several elevations into one geographic
/// surface, such as composite reflectivity.
///
/// Layout: `values[row * width + col]`, row 0 = north edge.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct CartesianField {
    label: String,
    unit: String,
    extent: GeoExtent,
    width: usize,
    height: usize,
    values: Vec<f32>,
    statuses: Vec<GateStatus>,
}

impl CartesianField {
    /// Create a new empty Cartesian field.
    ///
    /// All values are initialized to `0.0` with [`GateStatus::NoData`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidGeometry`] if `width` or `height` is zero or their product
    /// overflows, or the extent is not finite, in range, and ordered (`min < max`).
    ///
    /// # Example
    ///
    /// ```
    /// use nexrad_model::data::CartesianField;
    /// use nexrad_model::geo::{GeoExtent, GeoPoint};
    ///
    /// let extent = GeoExtent {
    ///     min: GeoPoint { latitude: 34.0, longitude: -99.0 },
    ///     max: GeoPoint { latitude: 36.0, longitude: -97.0 },
    /// };
    /// let field = CartesianField::new("Composite Reflectivity", "dBZ", extent, 200, 100)?;
    /// assert_eq!(field.get(0, 199).map(|(value, _)| value), Some(0.0));
    /// assert!(field.get(0, 200).is_none());
    /// # Ok::<(), nexrad_model::result::Error>(())
    /// ```
    pub fn new(
        label: impl Into<String>,
        unit: impl Into<String>,
        extent: GeoExtent,
        width: usize,
        height: usize,
    ) -> Result<Self> {
        let total = check_dimensions(("width", width), ("height", height))?;
        check_coordinate_range(
            "extent.latitude",
            (extent.min.latitude, extent.max.latitude),
            90.0,
            "must be within -90.0..=90.0",
        )?;
        check_coordinate_range(
            "extent.longitude",
            (extent.min.longitude, extent.max.longitude),
            180.0,
            "must be within -180.0..=180.0",
        )?;

        Ok(Self {
            label: label.into(),
            unit: unit.into(),
            extent,
            width,
            height,
            values: vec![0.0; total],
            statuses: vec![GateStatus::NoData; total],
        })
    }

    /// Human-readable label for this field.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Physical unit string.
    pub fn unit(&self) -> &str {
        &self.unit
    }

    /// Geographic extent of the grid.
    pub fn extent(&self) -> &GeoExtent {
        &self.extent
    }

    /// Number of columns.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Number of rows.
    pub fn height(&self) -> usize {
        self.height
    }

    /// All values as a flat slice.
    pub fn values(&self) -> &[f32] {
        &self.values
    }

    /// All statuses as a flat slice.
    pub fn statuses(&self) -> &[GateStatus] {
        &self.statuses
    }

    /// Get the value and status at a specific (row, col).
    ///
    /// Returns `None` if `row` is not less than [`height`](Self::height) or `col` is not less
    /// than [`width`](Self::width).
    pub fn get(&self, row: usize, col: usize) -> Option<(f32, GateStatus)> {
        let idx = flat_index(row, col, self.height, self.width)?;
        Some((*self.values.get(idx)?, *self.statuses.get(idx)?))
    }

    /// Set the value and status at a specific (row, col).
    ///
    /// # Errors
    ///
    /// Returns [`Error::IndexOutOfBounds`] if either index is outside the field's dimensions.
    pub fn set(&mut self, row: usize, col: usize, value: f32, status: GateStatus) -> Result<()> {
        set_cell(
            &mut self.values,
            &mut self.statuses,
            (row, col),
            (self.height, self.width),
            (value, status),
        )
    }

    /// Query the value at a geographic point.
    ///
    /// Returns `None` if the point is outside the field's extent.
    pub fn value_at_geo(&self, point: GeoPoint) -> Option<(f32, GateStatus)> {
        let GeoExtent { min, max } = self.extent;
        if !((min.latitude..=max.latitude).contains(&point.latitude)
            && (min.longitude..=max.longitude).contains(&point.longitude))
        {
            return None;
        }

        let lat_frac = (max.latitude - point.latitude) / (max.latitude - min.latitude);
        let lon_frac = (point.longitude - min.longitude) / (max.longitude - min.longitude);

        // A point on the south or east edge maps one past the last row or column.
        let row = ((lat_frac * self.height as f64) as usize).min(self.height - 1);
        let col = ((lon_frac * self.width as f64) as usize).min(self.width - 1);

        self.get(row, col)
    }

    /// Compute the (min, max) of valid values in this field.
    pub fn value_range(&self) -> Option<(f32, f32)> {
        valid_value_range(&self.values, &self.statuses)
    }
}

/// A 2D vertical cross-section grid of values.
///
/// Used for RHI (Range-Height Indicator) displays and arbitrary vertical slices through
/// a scan. The horizontal axis represents distance from the radar (or from the
/// start of a cross-section path) and the vertical axis represents altitude.
///
/// Layout: `values[row * width + col]`, row 0 = top (highest altitude).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct VerticalField {
    label: String,
    unit: String,
    distance_range_km: (f64, f64),
    altitude_range_meters: (f64, f64),
    width: usize,
    height: usize,
    values: Vec<f32>,
    statuses: Vec<GateStatus>,
}

impl VerticalField {
    /// Create a new empty vertical field.
    ///
    /// All values are initialized to `0.0` with [`GateStatus::NoData`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidGeometry`] if `width` or `height` is zero or their product
    /// overflows, or either range is not finite and ordered (`min < max`).
    ///
    /// # Example
    ///
    /// ```
    /// use nexrad_model::data::VerticalField;
    ///
    /// let field = VerticalField::new("Reflectivity", "dBZ", (0.0, 100.0), (0.0, 15_000.0), 400, 150)?;
    /// assert!(field.get(149, 399).is_some());
    /// assert!(field.get(150, 0).is_none());
    /// # Ok::<(), nexrad_model::result::Error>(())
    /// ```
    pub fn new(
        label: impl Into<String>,
        unit: impl Into<String>,
        distance_range_km: (f64, f64),
        altitude_range_meters: (f64, f64),
        width: usize,
        height: usize,
    ) -> Result<Self> {
        let total = check_dimensions(("width", width), ("height", height))?;
        check_ordered_range("distance_range_km", distance_range_km)?;
        check_ordered_range("altitude_range_meters", altitude_range_meters)?;

        Ok(Self {
            label: label.into(),
            unit: unit.into(),
            distance_range_km,
            altitude_range_meters,
            width,
            height,
            values: vec![0.0; total],
            statuses: vec![GateStatus::NoData; total],
        })
    }

    /// Human-readable label for this field.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Physical unit string.
    pub fn unit(&self) -> &str {
        &self.unit
    }

    /// Horizontal axis: (min_km, max_km) distance range.
    pub fn distance_range_km(&self) -> (f64, f64) {
        self.distance_range_km
    }

    /// Vertical axis: (min_meters, max_meters) altitude range.
    pub fn altitude_range_meters(&self) -> (f64, f64) {
        self.altitude_range_meters
    }

    /// Number of horizontal bins (columns).
    pub fn width(&self) -> usize {
        self.width
    }

    /// Number of vertical bins (rows).
    pub fn height(&self) -> usize {
        self.height
    }

    /// All values as a flat slice.
    pub fn values(&self) -> &[f32] {
        &self.values
    }

    /// All statuses as a flat slice.
    pub fn statuses(&self) -> &[GateStatus] {
        &self.statuses
    }

    /// Get the value and status at a specific (row, col).
    ///
    /// Returns `None` if `row` is not less than [`height`](Self::height) or `col` is not less
    /// than [`width`](Self::width).
    pub fn get(&self, row: usize, col: usize) -> Option<(f32, GateStatus)> {
        let idx = flat_index(row, col, self.height, self.width)?;
        Some((*self.values.get(idx)?, *self.statuses.get(idx)?))
    }

    /// Set the value and status at a specific (row, col).
    ///
    /// # Errors
    ///
    /// Returns [`Error::IndexOutOfBounds`] if either index is outside the field's dimensions.
    pub fn set(&mut self, row: usize, col: usize, value: f32, status: GateStatus) -> Result<()> {
        set_cell(
            &mut self.values,
            &mut self.statuses,
            (row, col),
            (self.height, self.width),
            (value, status),
        )
    }

    /// Query the value at a distance/altitude coordinate.
    ///
    /// Returns `None` if the coordinate is outside the field's range.
    pub fn value_at(&self, distance_km: f64, altitude_meters: f64) -> Option<(f32, GateStatus)> {
        let (d_min, d_max) = self.distance_range_km;
        let (a_min, a_max) = self.altitude_range_meters;

        if !((d_min..=d_max).contains(&distance_km) && (a_min..=a_max).contains(&altitude_meters)) {
            return None;
        }

        let col_frac = (distance_km - d_min) / (d_max - d_min);
        let row_frac = (a_max - altitude_meters) / (a_max - a_min); // Inverted: top = highest

        // A coordinate on the far distance or lowest altitude edge maps one past the last bin.
        let col = ((col_frac * self.width as f64) as usize).min(self.width - 1);
        let row = ((row_frac * self.height as f64) as usize).min(self.height - 1);

        self.get(row, col)
    }

    /// Compute the (min, max) of valid values in this field.
    pub fn value_range(&self) -> Option<(f32, f32)> {
        valid_value_range(&self.values, &self.statuses)
    }
}

/// The flat row-major index of `(row, col)`, if both are within the dimensions.
fn flat_index(row: usize, col: usize, rows: usize, columns: usize) -> Option<usize> {
    (row < rows && col < columns).then(|| row * columns + col)
}

/// Deserializes through [`SweepField::builder`], so the geometry is validated and `values` and
/// `statuses` must each hold one entry per gate.
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for SweepField {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let repr = SweepFieldRepr::deserialize(deserializer)?;
        let mut builder = SweepField::builder(repr.label, repr.unit)
            .elevation_angle_degrees(repr.elevation_angle_degrees)
            .azimuth_angles_degrees(repr.azimuth_angles_degrees)
            .azimuth_spacing_degrees(repr.azimuth_spacing_degrees)
            .first_gate_range_km(repr.first_gate_range_km)
            .gate_interval_km(repr.gate_interval_km)
            .gate_count(repr.gate_count);
        if let Some(product) = repr.product {
            builder = builder.product(product);
        }
        let mut field = builder.build().map_err(serde::de::Error::custom)?;
        (field.values, field.statuses) = cells(field.values.len(), repr.values, repr.statuses)?;
        Ok(field)
    }
}

/// The serialized form of [`SweepField`], validated before it becomes one.
#[cfg(feature = "serde")]
#[derive(Deserialize)]
#[serde(rename = "SweepField")]
struct SweepFieldRepr {
    label: String,
    unit: String,
    product: Option<Product>,
    elevation_angle_degrees: f32,
    azimuth_angles_degrees: Vec<f32>,
    azimuth_spacing_degrees: f32,
    first_gate_range_km: f64,
    gate_interval_km: f64,
    gate_count: usize,
    values: Vec<f32>,
    statuses: Vec<GateStatus>,
}

/// Deserializes through [`CartesianField::new`], so the dimensions and extent are validated and
/// `values` and `statuses` must each hold one entry per cell.
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for CartesianField {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let repr = CartesianFieldRepr::deserialize(deserializer)?;
        let mut field =
            CartesianField::new(repr.label, repr.unit, repr.extent, repr.width, repr.height)
                .map_err(serde::de::Error::custom)?;
        (field.values, field.statuses) = cells(field.values.len(), repr.values, repr.statuses)?;
        Ok(field)
    }
}

/// The serialized form of [`CartesianField`], validated before it becomes one.
#[cfg(feature = "serde")]
#[derive(Deserialize)]
#[serde(rename = "CartesianField")]
struct CartesianFieldRepr {
    label: String,
    unit: String,
    extent: GeoExtent,
    width: usize,
    height: usize,
    values: Vec<f32>,
    statuses: Vec<GateStatus>,
}

/// Deserializes through [`VerticalField::new`], so the dimensions and ranges are validated and
/// `values` and `statuses` must each hold one entry per cell.
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for VerticalField {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let repr = VerticalFieldRepr::deserialize(deserializer)?;
        let mut field = VerticalField::new(
            repr.label,
            repr.unit,
            repr.distance_range_km,
            repr.altitude_range_meters,
            repr.width,
            repr.height,
        )
        .map_err(serde::de::Error::custom)?;
        (field.values, field.statuses) = cells(field.values.len(), repr.values, repr.statuses)?;
        Ok(field)
    }
}

/// The serialized form of [`VerticalField`], validated before it becomes one.
#[cfg(feature = "serde")]
#[derive(Deserialize)]
#[serde(rename = "VerticalField")]
struct VerticalFieldRepr {
    label: String,
    unit: String,
    distance_range_km: (f64, f64),
    altitude_range_meters: (f64, f64),
    width: usize,
    height: usize,
    values: Vec<f32>,
    statuses: Vec<GateStatus>,
}

/// Check that deserialized value and status arrays each hold exactly `expected` cells.
#[cfg(feature = "serde")]
fn cells<E: serde::de::Error>(
    expected: usize,
    values: Vec<f32>,
    statuses: Vec<GateStatus>,
) -> std::result::Result<(Vec<f32>, Vec<GateStatus>), E> {
    for (name, len) in [("values", values.len()), ("statuses", statuses.len())] {
        if len != expected {
            return Err(E::custom(format_args!(
                "`{name}` holds {len} entries, expected {expected} for the field's dimensions"
            )));
        }
    }
    Ok((values, statuses))
}

/// Set one cell of parallel row-major value and status arrays.
fn set_cell(
    values: &mut [f32],
    statuses: &mut [GateStatus],
    (row, column): (usize, usize),
    (rows, columns): (usize, usize),
    (value, status): (f32, GateStatus),
) -> Result<()> {
    let out_of_bounds = Error::IndexOutOfBounds {
        row,
        column,
        rows,
        columns,
    };
    let idx = flat_index(row, column, rows, columns).ok_or(out_of_bounds.clone())?;
    match (values.get_mut(idx), statuses.get_mut(idx)) {
        (Some(value_slot), Some(status_slot)) => {
            *value_slot = value;
            *status_slot = status;
            Ok(())
        }
        _ => Err(out_of_bounds),
    }
}

/// The (min, max) of values whose status is [`GateStatus::Valid`].
fn valid_value_range(values: &[f32], statuses: &[GateStatus]) -> Option<(f32, f32)> {
    values
        .iter()
        .zip(statuses)
        .filter(|(_, &status)| status == GateStatus::Valid)
        .fold(None, |range, (&value, _)| match range {
            None => Some((value, value)),
            Some((min, max)) => Some((f32::min(min, value), f32::max(max, value))),
        })
}

fn invalid_geometry(parameter: &'static str, reason: &'static str) -> Error {
    Error::InvalidGeometry { parameter, reason }
}

/// Check that both dimensions are non-zero and their product fits in `usize`, returning it.
fn check_dimensions(
    (columns_name, columns): (&'static str, usize),
    (rows_name, rows): (&'static str, usize),
) -> Result<usize> {
    if columns == 0 {
        return Err(invalid_geometry(columns_name, "must be greater than zero"));
    }
    if rows == 0 {
        return Err(invalid_geometry(rows_name, "must be greater than zero"));
    }
    columns.checked_mul(rows).ok_or(invalid_geometry(
        columns_name,
        "times the row count overflows usize",
    ))
}

/// Check that a value is finite and greater than zero.
fn check_positive(parameter: &'static str, value: f64) -> Result<()> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(invalid_geometry(
            parameter,
            "must be finite and greater than zero",
        ))
    }
}

/// Check that a `(min, max)` range is finite and ordered.
fn check_ordered_range(parameter: &'static str, (min, max): (f64, f64)) -> Result<()> {
    if !(min.is_finite() && max.is_finite()) {
        return Err(invalid_geometry(parameter, "must be finite"));
    }
    if min >= max {
        return Err(invalid_geometry(parameter, "must have min less than max"));
    }
    Ok(())
}

/// Check that a `(min, max)` coordinate range is finite, ordered, and within `-limit..=limit`.
fn check_coordinate_range(
    parameter: &'static str,
    range: (f64, f64),
    limit: f64,
    out_of_range_reason: &'static str,
) -> Result<()> {
    check_ordered_range(parameter, range)?;
    let bounds = -limit..=limit;
    if bounds.contains(&range.0) && bounds.contains(&range.1) {
        Ok(())
    } else {
        Err(invalid_geometry(parameter, out_of_range_reason))
    }
}
