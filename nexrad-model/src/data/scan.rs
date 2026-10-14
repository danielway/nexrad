use crate::data::{Product, Sweep, SweepField, VcpNumber, VolumeCoveragePattern};
use crate::geo::RadarCoordinateSystem;
use crate::meta::Site;
use std::fmt::Display;

#[cfg(feature = "chrono")]
use chrono::{DateTime, Utc};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A single radar scan composed of a series of sweeps at different elevations. The pattern of
/// sweeps, including elevations and resolution, is determined by the scanning strategy of the
/// radar. This is referred to as the Volume Coverage Pattern (VCP).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Scan {
    site: Option<Site>,
    coverage_pattern: VolumeCoveragePattern,
    sweeps: Vec<Sweep>,
}

impl Scan {
    /// Create a new radar scan with the given coverage pattern and sweeps.
    pub fn new(coverage_pattern: VolumeCoveragePattern, sweeps: Vec<Sweep>) -> Self {
        Self {
            site: None,
            coverage_pattern,
            sweeps,
        }
    }

    /// Create a new radar scan with site metadata, coverage pattern, and sweeps.
    pub fn with_site(
        site: Site,
        coverage_pattern: VolumeCoveragePattern,
        sweeps: Vec<Sweep>,
    ) -> Self {
        Self {
            site: Some(site),
            coverage_pattern,
            sweeps,
        }
    }

    /// The radar site that produced this scan, if available.
    pub fn site(&self) -> Option<&Site> {
        self.site.as_ref()
    }

    /// This scan's coverage pattern number.
    pub fn coverage_pattern_number(&self) -> VcpNumber {
        self.coverage_pattern.pattern_number()
    }

    /// The coverage pattern configuration for this scan.
    ///
    /// This contains detailed configuration about how the radar performs the scan, including
    /// per-elevation settings, SAILS/MRLE/MPDA capabilities, and various thresholds.
    pub fn coverage_pattern(&self) -> &VolumeCoveragePattern {
        &self.coverage_pattern
    }

    /// The elevation sweeps comprising this scan.
    pub fn sweeps(&self) -> &[Sweep] {
        &self.sweeps
    }

    /// Extract a [`SweepField`] for `product` from every sweep that carries it.
    ///
    /// Sweeps without data for `product` are skipped, so the result may be shorter than
    /// [`sweeps`](Self::sweeps); the remaining fields are in scan order. See
    /// [`Sweep::field`].
    ///
    /// # Example
    ///
    /// ```
    /// use nexrad_model::data::{MomentData, Product, Radial, Scan, Sweep, VcpNumber,
    ///     VolumeCoveragePattern};
    ///
    /// let velocity = MomentData::from_fixed_point(
    ///     Product::Velocity, 2, 2000, 250, 8, 2.0, 129.0, vec![100, 140],
    /// )?;
    /// let with_velocity = Radial::builder(0, 0.0, 2, 0.9).moment(velocity).build()?;
    /// let without_velocity = Radial::builder(0, 0.0, 1, 0.5).build()?;
    /// let vcp = VolumeCoveragePattern::builder(VcpNumber::from_number(212)).build();
    /// let scan = Scan::new(
    ///     vcp,
    ///     vec![Sweep::new(1, vec![without_velocity])?, Sweep::new(2, vec![with_velocity])?],
    /// );
    ///
    /// let fields = scan.fields(Product::Velocity);
    /// assert_eq!(fields.len(), 1);
    /// # Ok::<(), nexrad_model::result::Error>(())
    /// ```
    pub fn fields(&self, product: Product) -> Vec<SweepField> {
        self.sweeps
            .iter()
            .filter_map(|sweep| sweep.field(product))
            .collect()
    }

    /// Extract a [`SweepField`] for `product` from the first sweep that carries it.
    ///
    /// Returns the index of that sweep within [`sweeps`](Self::sweeps) alongside the field,
    /// or `None` if no sweep has data for `product`.
    ///
    /// # Example
    ///
    /// ```
    /// use nexrad_model::data::{MomentData, Product, Radial, Scan, Sweep, VcpNumber,
    ///     VolumeCoveragePattern};
    ///
    /// let velocity = MomentData::from_fixed_point(
    ///     Product::Velocity, 2, 2000, 250, 8, 2.0, 129.0, vec![100, 140],
    /// )?;
    /// let with_velocity = Radial::builder(0, 0.0, 2, 0.9).moment(velocity).build()?;
    /// let without_velocity = Radial::builder(0, 0.0, 1, 0.5).build()?;
    /// let vcp = VolumeCoveragePattern::builder(VcpNumber::from_number(212)).build();
    /// let scan = Scan::new(
    ///     vcp,
    ///     vec![Sweep::new(1, vec![without_velocity])?, Sweep::new(2, vec![with_velocity])?],
    /// );
    ///
    /// let (index, field) = scan.first_field(Product::Velocity).unwrap();
    /// assert_eq!(index, 1);
    /// assert_eq!(field.gate_count(), 2);
    /// # Ok::<(), nexrad_model::result::Error>(())
    /// ```
    pub fn first_field(&self, product: Product) -> Option<(usize, SweepField)> {
        self.sweeps
            .iter()
            .enumerate()
            .find_map(|(index, sweep)| sweep.field(product).map(|field| (index, field)))
    }

    /// A [`RadarCoordinateSystem`] centered on this scan's radar site.
    ///
    /// Returns `None` if the scan has no [`site`](Self::site) metadata.
    ///
    /// # Example
    ///
    /// ```
    /// use nexrad_model::data::{Scan, VcpNumber, VolumeCoveragePattern};
    /// use nexrad_model::meta::Site;
    ///
    /// let vcp = VolumeCoveragePattern::builder(VcpNumber::from_number(212)).build();
    /// let site = Site::builder(*b"KTLX", 35.3331, -97.2778).build()?;
    ///
    /// let scan = Scan::with_site(site, vcp.clone(), vec![]);
    /// let coords = scan.coordinate_system().unwrap();
    /// assert_eq!(coords.latitude(), 35.3331);
    ///
    /// assert!(Scan::new(vcp, vec![]).coordinate_system().is_none());
    /// # Ok::<(), nexrad_model::result::Error>(())
    /// ```
    pub fn coordinate_system(&self) -> Option<RadarCoordinateSystem> {
        self.site().map(RadarCoordinateSystem::new)
    }

    /// All sweeps at the given elevation angle in degrees, in scan order.
    ///
    /// Uses a 0.05-degree tolerance for matching. SAILS and MRLE VCPs repeat certain
    /// elevation angles at multiple points during a scan, so this may yield more than
    /// one sweep.
    pub fn sweeps_at_elevation(
        &self,
        elevation_angle_degrees: f32,
    ) -> impl Iterator<Item = &Sweep> {
        self.sweeps.iter().filter(move |s| {
            s.elevation_angle_degrees()
                .map(|a| (a - elevation_angle_degrees).abs() < 0.05)
                .unwrap_or(false)
        })
    }

    /// The time range of this scan as `(earliest, latest)` collection times across all sweeps.
    ///
    /// Returns `None` if the scan has no radials or no valid timestamps.
    #[cfg(feature = "chrono")]
    pub fn time_range(&self) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
        let mut earliest: Option<DateTime<Utc>> = None;
        let mut latest: Option<DateTime<Utc>> = None;

        for sweep in &self.sweeps {
            if let Some((sweep_start, sweep_end)) = sweep.time_range() {
                earliest = Some(match earliest {
                    Some(e) => e.min(sweep_start),
                    None => sweep_start,
                });
                latest = Some(match latest {
                    Some(l) => l.max(sweep_end),
                    None => sweep_end,
                });
            }
        }

        earliest.zip(latest)
    }
}

impl Display for Scan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Scan ({}, {} sweeps)",
            self.coverage_pattern.pattern_number(),
            self.sweeps.len()
        )
    }
}
