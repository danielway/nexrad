use crate::data::{
    ElevationCut, Product, Sweep, SweepField, VcpNumber, VolumeCoveragePattern, WaveformType,
};
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
    /// Split cuts, SAILS, MRLE and MPDA repeat elevations within a volume, so reflectivity and
    /// velocity at one angle usually come from different sweeps and a product may appear
    /// several times. Use [`elevation_cut`](Self::elevation_cut) and
    /// [`sweeps_with_waveform`](Self::sweeps_with_waveform) to select sweeps by their cut.
    ///
    /// # Example
    ///
    /// ```
    /// use nexrad_model::data::{MomentData, Product, Radial, Scan, Sweep, VcpNumber,
    ///     VolumeCoveragePattern};
    ///
    /// let velocity = MomentData::builder(Product::Velocity, vec![100, 140])
    ///     .first_gate_range_meters(2000)
    ///     .gate_interval_meters(250)
    ///     .scale(2.0)
    ///     .offset(129.0)
    ///     .build()?;
    /// let with_velocity = Radial::builder(0, 0.0, 2, 0.9).moment(velocity).build()?;
    /// let without_velocity = Radial::builder(0, 0.0, 1, 0.5).build()?;
    /// let vcp = VolumeCoveragePattern::builder(VcpNumber::from_number(212)).build()?;
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
    /// Split cuts, SAILS, MRLE and MPDA repeat elevations within a volume, so reflectivity and
    /// velocity at one angle usually come from different sweeps and a product may appear
    /// several times. Use [`elevation_cut`](Self::elevation_cut) and
    /// [`sweeps_with_waveform`](Self::sweeps_with_waveform) to select sweeps by their cut.
    ///
    /// # Example
    ///
    /// ```
    /// use nexrad_model::data::{MomentData, Product, Radial, Scan, Sweep, VcpNumber,
    ///     VolumeCoveragePattern};
    ///
    /// let velocity = MomentData::builder(Product::Velocity, vec![100, 140])
    ///     .first_gate_range_meters(2000)
    ///     .gate_interval_meters(250)
    ///     .scale(2.0)
    ///     .offset(129.0)
    ///     .build()?;
    /// let with_velocity = Radial::builder(0, 0.0, 2, 0.9).moment(velocity).build()?;
    /// let without_velocity = Radial::builder(0, 0.0, 1, 0.5).build()?;
    /// let vcp = VolumeCoveragePattern::builder(VcpNumber::from_number(212)).build()?;
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
    /// let vcp = VolumeCoveragePattern::builder(VcpNumber::from_number(212)).build()?;
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
    /// Uses a 0.05-degree tolerance for matching. Split cuts, SAILS, MRLE and MPDA repeat
    /// elevations within a volume, so this may yield several sweeps, and reflectivity and
    /// velocity at one angle usually come from different ones. Use
    /// [`elevation_cut`](Self::elevation_cut) and
    /// [`sweeps_with_waveform`](Self::sweeps_with_waveform) to select sweeps by their cut.
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

    /// The coverage pattern's elevation cut that `sweep` was collected with.
    ///
    /// The cut is the one whose 1-based position in
    /// [`VolumeCoveragePattern::elevation_cuts`] equals [`Sweep::elevation_number`]. Returns
    /// `None` when the elevation number is zero or beyond the pattern's cuts.
    ///
    /// # Example
    ///
    /// ```
    /// use nexrad_model::data::{ChannelConfiguration, ElevationCut, Radial, Scan, Sweep,
    ///     VcpNumber, VolumeCoveragePattern, WaveformType};
    ///
    /// let surveillance = ElevationCut::builder(
    ///     0.5, WaveformType::ContiguousSurveillance, ChannelConfiguration::ConstantPhase,
    /// ).build()?;
    /// let vcp = VolumeCoveragePattern::builder(VcpNumber::Vcp212)
    ///     .elevation_cut(surveillance)
    ///     .build()?;
    /// let sweep = Sweep::new(1, vec![Radial::builder(0, 0.0, 1, 0.5).build()?])?;
    /// let scan = Scan::new(vcp, vec![sweep]);
    ///
    /// let cut = scan.elevation_cut(&scan.sweeps()[0]).unwrap();
    /// assert_eq!(cut.waveform_type(), WaveformType::ContiguousSurveillance);
    /// # Ok::<(), nexrad_model::result::Error>(())
    /// ```
    pub fn elevation_cut(&self, sweep: &Sweep) -> Option<&ElevationCut> {
        let index = usize::from(sweep.elevation_number()).checked_sub(1)?;
        self.coverage_pattern.elevation_cuts().get(index)
    }

    /// The sweeps whose [`elevation_cut`](Self::elevation_cut) uses `waveform_type`, in scan
    /// order.
    ///
    /// Sweeps without a matching cut in the coverage pattern are skipped. Selecting
    /// [`WaveformType::ContiguousSurveillance`] yields the long-PRT surveillance halves of split
    /// cuts, which carry reflectivity with the greatest unambiguous range, while the Doppler
    /// waveforms yield the sweeps carrying velocity.
    pub fn sweeps_with_waveform(
        &self,
        waveform_type: WaveformType,
    ) -> impl Iterator<Item = &Sweep> {
        self.sweeps.iter().filter(move |sweep| {
            self.elevation_cut(sweep)
                .is_some_and(|cut| cut.waveform_type() == waveform_type)
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
