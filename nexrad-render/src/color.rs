//! Color scales for radar data visualization.
//!
//! This module provides types for mapping radar moment values to colors.
//! The primary type is [`DiscreteColorScale`], which maps value ranges to
//! specific colors based on threshold levels. For rendering, the scale is
//! converted to a [`ColorLookupTable`] which provides O(1) color lookups.

use crate::{Error, Result};
use nexrad_model::data::Product;

/// An RGBA color with components in the range `0.0..=1.0`.
///
/// This type is used for defining color scales and render backgrounds. Colors are
/// converted to 8-bit RGBA values during rendering. The default color is
/// [`Color::TRANSPARENT`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    /// Red component, `0.0..=1.0`.
    pub r: f64,
    /// Green component, `0.0..=1.0`.
    pub g: f64,
    /// Blue component, `0.0..=1.0`.
    pub b: f64,
    /// Alpha component, `0.0..=1.0` (0 is fully transparent).
    pub a: f64,
}

impl Color {
    /// Creates a new color from RGB components (alpha defaults to 1.0).
    ///
    /// Components should be in the range `0.0..=1.0`.
    pub const fn rgb(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Creates a new color from RGBA components.
    ///
    /// Components should be in the range `0.0..=1.0`.
    pub const fn rgba(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }

    /// Creates a color from 8-bit RGBA values, mapping `0..=255` to `0.0..=1.0`.
    ///
    /// [`to_rgba8`](Self::to_rgba8) converts it back to the same bytes.
    pub fn from_rgba8(rgba: [u8; 4]) -> Self {
        let [r, g, b, a] = rgba.map(|c| f64::from(c) / 255.0);
        Self { r, g, b, a }
    }

    /// Converts the color to 8-bit RGBA values.
    ///
    /// Each component is clamped to `0.0..=1.0` and rounded to the nearest byte value;
    /// a NaN component becomes 0.
    pub fn to_rgba8(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a].map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8)
    }

    /// Black color.
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);

    /// White color.
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);

    /// Transparent (fully transparent black).
    pub const TRANSPARENT: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);
}

impl Default for Color {
    /// [`Color::TRANSPARENT`].
    fn default() -> Self {
        Self::TRANSPARENT
    }
}

impl From<[u8; 4]> for Color {
    /// See [`Color::from_rgba8`].
    fn from(rgba: [u8; 4]) -> Self {
        Self::from_rgba8(rgba)
    }
}

impl From<Color> for [u8; 4] {
    /// See [`Color::to_rgba8`].
    fn from(color: Color) -> Self {
        color.to_rgba8()
    }
}

/// A single level in a discrete color scale.
///
/// Represents a threshold value and its associated color. Values at or above
/// this threshold (but below the next higher threshold) will be rendered with
/// this color.
#[derive(Debug, Clone)]
pub struct ColorScaleLevel {
    /// The threshold value.
    pub value: f32,
    /// The color for values at or above this threshold.
    pub color: Color,
}

impl ColorScaleLevel {
    /// Creates a new color scale level.
    ///
    /// # Arguments
    ///
    /// * `value` - The threshold value
    /// * `color` - The color to use for values at or above this threshold
    pub fn new(value: f32, color: Color) -> Self {
        Self { value, color }
    }
}

/// A discrete color scale that maps value ranges to colors.
///
/// The scale works by finding the highest threshold that the input value
/// exceeds, and returning the corresponding color. Levels are automatically
/// sorted from highest to lowest threshold during construction.
///
/// # Example
///
/// ```
/// use nexrad_render::{ColorScaleLevel, DiscreteColorScale, Color};
///
/// let scale = DiscreteColorScale::new(vec![
///     ColorScaleLevel::new(0.0, Color::BLACK),
///     ColorScaleLevel::new(30.0, Color::rgb(0.0, 1.0, 0.0)),
///     ColorScaleLevel::new(50.0, Color::rgb(1.0, 0.0, 0.0)),
/// ])?;
///
/// // Values >= 50 return red, >= 30 return green, >= 0 return black
/// assert_eq!(scale.color(40.0), Color::rgb(0.0, 1.0, 0.0));
///
/// // A scale needs at least one level, and every threshold must be finite.
/// assert!(DiscreteColorScale::new(vec![]).is_err());
/// assert!(DiscreteColorScale::new(vec![ColorScaleLevel::new(f32::NAN, Color::BLACK)]).is_err());
/// # Ok::<(), nexrad_render::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct DiscreteColorScale {
    levels: Vec<ColorScaleLevel>,
}

impl DiscreteColorScale {
    /// Creates a new discrete color scale from the given levels.
    ///
    /// Levels may be given in any order and are sorted from highest to lowest threshold. Two
    /// levels may share a threshold; the one that comes first in `levels` is drawn and the other
    /// is not. Colors are not checked.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidColorScale`] if `levels` is empty or a threshold is not finite.
    pub fn new(levels: Vec<ColorScaleLevel>) -> Result<Self> {
        check_scale_values(levels.iter().map(|level| level.value))?;
        Ok(Self::from_levels(levels))
    }

    /// Builds a scale from levels known to be valid, as the built-in scales are.
    fn from_levels(mut levels: Vec<ColorScaleLevel>) -> Self {
        levels.sort_by(|a, b| b.value.total_cmp(&a.value));
        Self { levels }
    }

    /// Returns the color for the given value.
    ///
    /// Finds the highest threshold that the value exceeds and returns its color.
    /// If the value is below all thresholds, returns the color of the lowest threshold.
    pub fn color(&self, value: f32) -> Color {
        let mut color = Color::BLACK;

        for level in &self.levels {
            if value >= level.value {
                return level.color;
            }

            color = level.color;
        }

        color
    }

    /// Returns the levels in this color scale (sorted highest to lowest), for
    /// example to draw a legend.
    pub fn levels(&self) -> &[ColorScaleLevel] {
        &self.levels
    }
}

/// A color stop for a continuous color scale.
///
/// Defines a value-to-color control point. Colors are linearly interpolated
/// between stops.
#[derive(Debug, Clone)]
pub struct ColorStop {
    /// The value at this stop.
    pub value: f32,
    /// The color at this stop.
    pub color: Color,
}

impl ColorStop {
    /// Creates a new color stop.
    pub fn new(value: f32, color: Color) -> Self {
        Self { value, color }
    }
}

/// A continuous color scale that linearly interpolates between color stops.
///
/// Unlike [`DiscreteColorScale`], this scale produces smooth color gradients
/// between control points. This is useful for products where smooth transitions
/// are more informative than discrete steps.
///
/// # Example
///
/// ```
/// use nexrad_render::{ColorStop, ContinuousColorScale, Color};
///
/// let scale = ContinuousColorScale::new(vec![
///     ColorStop::new(0.0, Color::rgb(0.0, 0.0, 1.0)),   // Blue at 0
///     ColorStop::new(50.0, Color::rgb(0.0, 1.0, 0.0)),  // Green at 50
///     ColorStop::new(100.0, Color::rgb(1.0, 0.0, 0.0)), // Red at 100
/// ])?;
///
/// // Value 25 produces a blue-green blend
/// let color = scale.color(25.0);
/// # Ok::<(), nexrad_render::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct ContinuousColorScale {
    stops: Vec<ColorStop>,
}

impl ContinuousColorScale {
    /// Creates a new continuous color scale from color stops.
    ///
    /// Stops may be given in any order and are sorted by value (lowest to highest). Two stops
    /// may share a value, which makes a hard step between their colors. Colors are not checked.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidColorScale`] if `stops` is empty or a value is not finite.
    pub fn new(mut stops: Vec<ColorStop>) -> Result<Self> {
        check_scale_values(stops.iter().map(|stop| stop.value))?;
        stops.sort_by(|a, b| a.value.total_cmp(&b.value));
        Ok(Self { stops })
    }

    /// Returns the linearly interpolated color for the given value.
    ///
    /// Values below the lowest stop get the lowest stop's color.
    /// Values above the highest stop get the highest stop's color.
    pub fn color(&self, value: f32) -> Color {
        if self.stops.is_empty() {
            return Color::BLACK;
        }

        if value <= self.stops[0].value {
            return self.stops[0].color;
        }

        let last = self.stops.len() - 1;
        if value >= self.stops[last].value {
            return self.stops[last].color;
        }

        // Find the two stops that bracket this value
        for i in 0..last {
            let low = &self.stops[i];
            let high = &self.stops[i + 1];

            if value >= low.value && value <= high.value {
                let range = high.value - low.value;
                if range == 0.0 {
                    return low.color;
                }
                let t = ((value - low.value) / range) as f64;
                return Color::rgba(
                    low.color.r + (high.color.r - low.color.r) * t,
                    low.color.g + (high.color.g - low.color.g) * t,
                    low.color.b + (high.color.b - low.color.b) * t,
                    low.color.a + (high.color.a - low.color.a) * t,
                );
            }
        }

        self.stops[last].color
    }

    /// Returns the stops in this color scale (sorted lowest to highest).
    pub fn stops(&self) -> &[ColorStop] {
        &self.stops
    }
}

/// Checks the values of a color scale's levels or stops: at least one, and all finite.
fn check_scale_values(values: impl Iterator<Item = f32>) -> Result<()> {
    let mut count = 0;
    for value in values {
        if !value.is_finite() {
            return Err(Error::InvalidColorScale {
                reason: "every value must be finite",
            });
        }
        count += 1;
    }
    if count == 0 {
        return Err(Error::InvalidColorScale {
            reason: "must have at least one level or stop",
        });
    }
    Ok(())
}

/// A color scale that can be either discrete or continuous.
///
/// This enum allows rendering functions to accept either type of color scale. It is
/// non-exhaustive so that another kind of scale can be added in a minor release; a `match`
/// outside this crate needs a wildcard arm, and [`color`](Self::color) works for every kind.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ColorScale {
    /// A discrete step-function color scale.
    Discrete(DiscreteColorScale),
    /// A continuous linear-interpolation color scale.
    Continuous(ContinuousColorScale),
}

impl ColorScale {
    /// Returns the color for the given value.
    pub fn color(&self, value: f32) -> Color {
        match self {
            ColorScale::Discrete(scale) => scale.color(value),
            ColorScale::Continuous(scale) => scale.color(value),
        }
    }
}

impl From<DiscreteColorScale> for ColorScale {
    fn from(scale: DiscreteColorScale) -> Self {
        ColorScale::Discrete(scale)
    }
}

impl From<ContinuousColorScale> for ColorScale {
    fn from(scale: ContinuousColorScale) -> Self {
        ColorScale::Continuous(scale)
    }
}

/// A pre-computed lookup table for O(1) color lookups.
///
/// The table maps a value range to RGBA colors. It is created from a [`ColorScale`]
/// and provides fast color lookups during rendering:
///
/// - For a [`DiscreteColorScale`], [`color`](Self::color) returns exactly the scale's
///   color for any value in range, including values on a threshold, whatever the
///   table size.
/// - For a [`ContinuousColorScale`], [`color`](Self::color) returns the nearest of the
///   [`entries`](Self::entries).
///
/// # Example
///
/// ```
/// use nexrad_model::data::Product;
/// use nexrad_render::{default_scale, ColorLookupTable};
///
/// let scale = default_scale(Product::Reflectivity);
/// let lut = ColorLookupTable::new(&scale, -32.0, 95.0, 256)?;
///
/// // O(1) lookup returning [R, G, B, A] bytes
/// let color = lut.color(45.0);
/// assert_eq!(color, scale.color(45.0).to_rgba8());
///
/// // The raw table, e.g. for uploading as a 1-D GPU texture
/// assert_eq!(lut.entries().len(), lut.len());
/// # Ok::<(), nexrad_render::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct ColorLookupTable {
    /// RGBA colors of the scale sampled at evenly spaced values from min to max.
    table: Vec<[u8; 4]>,
    /// Minimum value in the mapped range.
    min_value: f32,
    /// Maximum value in the mapped range.
    max_value: f32,
    /// Exact step lookup for discrete scales; `None` for continuous scales.
    steps: Option<Steps>,
}

/// Exact lookup of a discrete scale's step function over a value range.
///
/// The range is split into equal buckets. `bucket_first[b]` is the number of
/// thresholds whose bucket is before `b`; every such threshold is below any value in
/// bucket `b` because bucketing is monotonic, so a lookup only compares the value with
/// the thresholds inside its own bucket. The bucket count is chosen so that buckets
/// usually hold at most one threshold.
#[derive(Debug, Clone)]
struct Steps {
    /// Distinct thresholds inside `(min, max]`, ascending.
    thresholds: Vec<f32>,
    /// `colors[0]` is the color at `min`; `colors[i + 1]` the color from `thresholds[i]`.
    colors: Vec<[u8; 4]>,
    /// Number of thresholds in buckets before each bucket.
    bucket_first: Vec<usize>,
}

/// Upper bound on the bucket count of a discrete lookup.
const MAX_BUCKETS: usize = 4096;

impl ColorLookupTable {
    /// Creates a lookup table by sampling a color scale.
    ///
    /// # Arguments
    ///
    /// * `scale` - The color scale to sample from (discrete or continuous)
    /// * `min` - The value mapped to the first entry
    /// * `max` - The value mapped to the last entry
    /// * `size` - The number of entries in the lookup table (256 recommended)
    ///
    /// Entry `i` holds the scale's color at `min + i / (size - 1) * (max - min)`.
    /// Values outside the `min..=max` range take the color at the nearer end.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidRange`] unless `min` and `max` are finite and
    /// `min < max`, and [`Error::InvalidTableSize`] if `size` is below 2.
    pub fn new(scale: &ColorScale, min: f32, max: f32, size: usize) -> Result<Self> {
        if !(min.is_finite() && max.is_finite() && min < max) {
            return Err(Error::InvalidRange { min, max });
        }
        if size < 2 {
            return Err(Error::InvalidTableSize { size });
        }

        let range = f64::from(max) - f64::from(min);
        let table = (0..size)
            .map(|i| {
                let value = (f64::from(min) + i as f64 / (size - 1) as f64 * range) as f32;
                scale.color(value.clamp(min, max)).to_rgba8()
            })
            .collect();

        let steps = match scale {
            ColorScale::Discrete(discrete) => Some(Steps::new(discrete, min, max, size)),
            ColorScale::Continuous(_) => None,
        };

        Ok(Self {
            table,
            min_value: min,
            max_value: max,
            steps,
        })
    }

    /// Returns the RGBA color for the given value.
    ///
    /// This is an O(1) operation. Values outside the table's range take the color at
    /// the nearer end; NaN takes the color at the minimum.
    #[inline]
    pub fn color(&self, value: f32) -> [u8; 4] {
        let value = if value.is_nan() {
            self.min_value
        } else {
            value.clamp(self.min_value, self.max_value)
        };
        match &self.steps {
            Some(steps) => steps.color(value, self.min_value, self.max_value),
            None => {
                let last = self.table.len() - 1;
                let index = (self.fraction(value) * last as f32).round() as usize;
                self.table[index.min(last)]
            }
        }
    }

    /// Position of an in-range value within the range, `0.0..=1.0`.
    #[inline]
    fn fraction(&self, value: f32) -> f32 {
        (value - self.min_value) / (self.max_value - self.min_value)
    }

    /// The table's RGBA entries: the scale sampled from
    /// [`min_value`](Self::min_value) to [`max_value`](Self::max_value) in equal
    /// steps.
    ///
    /// Suitable for uploading as a 1-D texture to apply the scale on a GPU.
    pub fn entries(&self) -> &[[u8; 4]] {
        &self.table
    }

    /// The value mapped to the first entry.
    pub fn min_value(&self) -> f32 {
        self.min_value
    }

    /// The value mapped to the last entry.
    pub fn max_value(&self) -> f32 {
        self.max_value
    }

    /// The number of entries in the table (always at least 2).
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Always `false`: a table has at least 2 entries.
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }
}

impl Steps {
    fn new(scale: &DiscreteColorScale, min: f32, max: f32, size: usize) -> Self {
        let mut thresholds: Vec<f32> = scale
            .levels()
            .iter()
            .map(|level| level.value)
            .filter(|&value| value > min && value <= max)
            .collect();
        thresholds.sort_by(f32::total_cmp);
        thresholds.dedup();

        let colors = std::iter::once(min)
            .chain(thresholds.iter().copied())
            .map(|value| scale.color(value).to_rgba8())
            .collect();

        // Enough buckets that thresholds rarely share one.
        let min_gap = thresholds
            .windows(2)
            .map(|pair| pair[1] - pair[0])
            .fold(max - min, f32::min);
        let wanted = ((max - min) / min_gap).ceil() as usize + 1;
        let buckets = wanted.clamp(size, MAX_BUCKETS.max(size));

        let mut bucket_first = vec![0; buckets];
        let mut next = 0;
        for (bucket, first) in bucket_first.iter_mut().enumerate() {
            while next < thresholds.len()
                && Self::bucket(thresholds[next], min, max, buckets) < bucket
            {
                next += 1;
            }
            *first = next;
        }

        Self {
            thresholds,
            colors,
            bucket_first,
        }
    }

    /// The bucket of an in-range value; monotonic in `value`.
    #[inline]
    fn bucket(value: f32, min: f32, max: f32, buckets: usize) -> usize {
        let index = ((value - min) / (max - min) * buckets as f32) as usize;
        index.min(buckets - 1)
    }

    #[inline]
    fn color(&self, value: f32, min: f32, max: f32) -> [u8; 4] {
        let bucket = Self::bucket(value, min, max, self.bucket_first.len());
        let mut passed = self.bucket_first[bucket];
        while self
            .thresholds
            .get(passed)
            .is_some_and(|&threshold| value >= threshold)
        {
            passed += 1;
        }
        self.colors[passed]
    }
}

/// Returns a discrete reflectivity color scale.
///
/// The scale steps every 5 dBZ from black at 0 dBZ through cyan and blue, greens,
/// yellow and orange, reds and purple to magenta and white at 75 dBZ and above.
/// Values below 0 dBZ take the first color.
///
/// | dBZ Range | Color |
/// |-----------|-------|
/// | 0-5 | Black |
/// | 5-20 | Cyan/Blue |
/// | 20-35 | Green |
/// | 35-50 | Yellow/Orange |
/// | 50-65 | Red/Magenta |
/// | 65+ | Purple/White |
pub fn reflectivity_scale() -> DiscreteColorScale {
    DiscreteColorScale::from_levels(vec![
        ColorScaleLevel::new(0.0, Color::rgb(0.0000, 0.0000, 0.0000)),
        ColorScaleLevel::new(5.0, Color::rgb(0.0000, 1.0000, 1.0000)),
        ColorScaleLevel::new(10.0, Color::rgb(0.5294, 0.8078, 0.9216)),
        ColorScaleLevel::new(15.0, Color::rgb(0.0000, 0.0000, 1.0000)),
        ColorScaleLevel::new(20.0, Color::rgb(0.0000, 1.0000, 0.0000)),
        ColorScaleLevel::new(25.0, Color::rgb(0.1961, 0.8039, 0.1961)),
        ColorScaleLevel::new(30.0, Color::rgb(0.1333, 0.5451, 0.1333)),
        ColorScaleLevel::new(35.0, Color::rgb(0.9333, 0.9333, 0.0000)),
        ColorScaleLevel::new(40.0, Color::rgb(0.9333, 0.8627, 0.5098)),
        ColorScaleLevel::new(45.0, Color::rgb(0.9333, 0.4627, 0.1294)),
        ColorScaleLevel::new(50.0, Color::rgb(1.0000, 0.1882, 0.1882)),
        ColorScaleLevel::new(55.0, Color::rgb(0.6902, 0.1882, 0.3765)),
        ColorScaleLevel::new(60.0, Color::rgb(0.6902, 0.1882, 0.3765)),
        ColorScaleLevel::new(65.0, Color::rgb(0.7294, 0.3333, 0.8275)),
        ColorScaleLevel::new(70.0, Color::rgb(1.0000, 0.0000, 1.0000)),
        ColorScaleLevel::new(75.0, Color::rgb(1.0000, 1.0000, 1.0000)),
    ])
}

/// Returns a color scale for radial velocity data.
///
/// This divergent scale uses greens for negative values (toward the radar) and
/// reds for positive values (away from the radar), with gray near zero.
/// Range: -64 to +64 m/s, which covers the -63.5 to +63.0 m/s that an 8-bit
/// velocity value encodes at 0.5 m/s resolution (ICD 2620002). Data encoded at
/// 1.0 m/s resolution can exceed it and takes the end colors.
///
/// | Velocity (m/s) | Color |
/// |----------------|-------|
/// | -64 to -48 | Dark Green |
/// | -48 to -32 | Green |
/// | -32 to -16 | Light Green |
/// | -16 to -4 | Pale Green |
/// | -4 to +4 | Gray |
/// | +4 to +16 | Pale Red |
/// | +16 to +32 | Light Red/Pink |
/// | +32 to +48 | Red |
/// | +48 to +64 | Dark Red |
pub fn velocity_scale() -> DiscreteColorScale {
    DiscreteColorScale::from_levels(vec![
        // Strong inbound (toward radar) - dark green
        ColorScaleLevel::new(-64.0, Color::rgb(0.0000, 0.3922, 0.0000)),
        ColorScaleLevel::new(-48.0, Color::rgb(0.0000, 0.5451, 0.0000)),
        ColorScaleLevel::new(-32.0, Color::rgb(0.0000, 0.8039, 0.0000)),
        ColorScaleLevel::new(-16.0, Color::rgb(0.5647, 0.9333, 0.5647)),
        // Near zero - gray
        ColorScaleLevel::new(-4.0, Color::rgb(0.6627, 0.6627, 0.6627)),
        ColorScaleLevel::new(4.0, Color::rgb(0.6627, 0.6627, 0.6627)),
        // Outbound (away from radar) - reds
        ColorScaleLevel::new(16.0, Color::rgb(1.0000, 0.7529, 0.7961)),
        ColorScaleLevel::new(32.0, Color::rgb(1.0000, 0.4118, 0.4118)),
        ColorScaleLevel::new(48.0, Color::rgb(0.8039, 0.0000, 0.0000)),
        ColorScaleLevel::new(64.0, Color::rgb(0.5451, 0.0000, 0.0000)),
    ])
}

/// Returns a color scale for spectrum width data.
///
/// This sequential scale runs from gray and cool colors for small widths to warm
/// colors for large widths. Range: 0 to 30 m/s.
///
/// | Width (m/s) | Color |
/// |-------------|-------|
/// | 0-4 | Gray |
/// | 4-8 | Blue |
/// | 8-12 | Cyan |
/// | 12-16 | Green |
/// | 16-20 | Yellow |
/// | 20-25 | Orange |
/// | 25-30 | Red |
pub fn spectrum_width_scale() -> DiscreteColorScale {
    DiscreteColorScale::from_levels(vec![
        ColorScaleLevel::new(0.0, Color::rgb(0.5020, 0.5020, 0.5020)),
        ColorScaleLevel::new(4.0, Color::rgb(0.0000, 0.0000, 0.8039)),
        ColorScaleLevel::new(8.0, Color::rgb(0.0000, 0.8039, 0.8039)),
        ColorScaleLevel::new(12.0, Color::rgb(0.0000, 0.8039, 0.0000)),
        ColorScaleLevel::new(16.0, Color::rgb(0.9333, 0.9333, 0.0000)),
        ColorScaleLevel::new(20.0, Color::rgb(1.0000, 0.6471, 0.0000)),
        ColorScaleLevel::new(25.0, Color::rgb(1.0000, 0.0000, 0.0000)),
    ])
}

/// Returns a color scale for differential reflectivity (ZDR) data.
///
/// This divergent scale shows negative values in purple and blue, values near
/// zero in gray, and positive values in green, yellow, orange and red.
/// Range: -2 to +6 dB.
///
/// | ZDR (dB) | Color |
/// |----------|-------|
/// | -2 to -1 | Purple |
/// | -1 to 0 | Blue |
/// | 0 to 0.5 | Gray |
/// | 0.5 to 1.5 | Light Green |
/// | 1.5 to 2.5 | Yellow |
/// | 2.5 to 4 | Orange |
/// | 4 to 6 | Red |
pub fn differential_reflectivity_scale() -> DiscreteColorScale {
    DiscreteColorScale::from_levels(vec![
        // Negative
        ColorScaleLevel::new(-2.0, Color::rgb(0.5020, 0.0000, 0.5020)),
        ColorScaleLevel::new(-1.0, Color::rgb(0.0000, 0.0000, 0.8039)),
        // Near zero
        ColorScaleLevel::new(0.0, Color::rgb(0.6627, 0.6627, 0.6627)),
        // Positive
        ColorScaleLevel::new(0.5, Color::rgb(0.5647, 0.9333, 0.5647)),
        ColorScaleLevel::new(1.5, Color::rgb(0.9333, 0.9333, 0.0000)),
        ColorScaleLevel::new(2.5, Color::rgb(1.0000, 0.6471, 0.0000)),
        ColorScaleLevel::new(4.0, Color::rgb(1.0000, 0.0000, 0.0000)),
    ])
}

/// Returns a color scale for correlation coefficient (CC/RhoHV) data.
///
/// This sequential scale uses narrower steps above 0.85, so values close to 1.0
/// are told apart more finely than low values. Range: 0.0 to 1.0; values above
/// 1.0 take the last color.
///
/// | CC | Color |
/// |----|-------|
/// | 0.0-0.7 | Black/Purple/Blue |
/// | 0.7-0.85 | Cyan/Teal |
/// | 0.85-0.92 | Light Green |
/// | 0.92-0.96 | Green |
/// | 0.96-0.98 | Yellow |
/// | 0.98-1.0 | White/Light Gray |
pub fn correlation_coefficient_scale() -> DiscreteColorScale {
    DiscreteColorScale::from_levels(vec![
        ColorScaleLevel::new(0.0, Color::rgb(0.0000, 0.0000, 0.0000)),
        ColorScaleLevel::new(0.2, Color::rgb(0.3922, 0.0000, 0.5882)),
        ColorScaleLevel::new(0.5, Color::rgb(0.0000, 0.0000, 0.8039)),
        ColorScaleLevel::new(0.7, Color::rgb(0.0000, 0.5451, 0.5451)),
        ColorScaleLevel::new(0.85, Color::rgb(0.0000, 0.8039, 0.4000)),
        ColorScaleLevel::new(0.92, Color::rgb(0.0000, 0.8039, 0.0000)),
        ColorScaleLevel::new(0.96, Color::rgb(0.9333, 0.9333, 0.0000)),
        ColorScaleLevel::new(0.98, Color::rgb(0.9020, 0.9020, 0.9020)),
    ])
}

/// Returns a color scale for differential phase (PhiDP) data.
///
/// This sequential scale covers the full 0-360 degree range in 45 degree steps.
/// Range: 0 to 360 degrees.
///
/// | PhiDP (deg) | Color |
/// |-------------|-------|
/// | 0-45 | Purple |
/// | 45-90 | Blue |
/// | 90-135 | Cyan |
/// | 135-180 | Green |
/// | 180-225 | Yellow |
/// | 225-270 | Orange |
/// | 270-315 | Red |
/// | 315-360 | Magenta |
pub fn differential_phase_scale() -> DiscreteColorScale {
    DiscreteColorScale::from_levels(vec![
        ColorScaleLevel::new(0.0, Color::rgb(0.5020, 0.0000, 0.5020)),
        ColorScaleLevel::new(45.0, Color::rgb(0.0000, 0.0000, 0.8039)),
        ColorScaleLevel::new(90.0, Color::rgb(0.0000, 0.8039, 0.8039)),
        ColorScaleLevel::new(135.0, Color::rgb(0.0000, 0.8039, 0.0000)),
        ColorScaleLevel::new(180.0, Color::rgb(0.9333, 0.9333, 0.0000)),
        ColorScaleLevel::new(225.0, Color::rgb(1.0000, 0.6471, 0.0000)),
        ColorScaleLevel::new(270.0, Color::rgb(1.0000, 0.0000, 0.0000)),
        ColorScaleLevel::new(315.0, Color::rgb(1.0000, 0.0000, 1.0000)),
    ])
}

/// Returns a color scale for clutter filter power (CFP) data.
///
/// This divergent scale centers around 0 dB, showing negative values
/// (filtered reflectivity lower than unfiltered) in blues and positive
/// values in reds.
/// Default range: -20 to +20 dB.
///
/// | CFP (dB) | Color |
/// |----------|-------|
/// | -20 to -10 | Dark Blue |
/// | -10 to -5 | Blue |
/// | -5 to -1 | Light Blue |
/// | -1 to +1 | Gray |
/// | +1 to +5 | Light Red |
/// | +5 to +10 | Red |
/// | +10 to +20 | Dark Red |
pub fn clutter_filter_power_scale() -> DiscreteColorScale {
    DiscreteColorScale::from_levels(vec![
        ColorScaleLevel::new(-20.0, Color::rgb(0.0000, 0.0000, 0.5451)),
        ColorScaleLevel::new(-10.0, Color::rgb(0.0000, 0.0000, 0.8039)),
        ColorScaleLevel::new(-5.0, Color::rgb(0.6784, 0.8471, 0.9020)),
        ColorScaleLevel::new(-1.0, Color::rgb(0.6627, 0.6627, 0.6627)),
        ColorScaleLevel::new(1.0, Color::rgb(0.6627, 0.6627, 0.6627)),
        ColorScaleLevel::new(5.0, Color::rgb(1.0000, 0.7529, 0.7961)),
        ColorScaleLevel::new(10.0, Color::rgb(1.0000, 0.4118, 0.4118)),
        ColorScaleLevel::new(20.0, Color::rgb(0.8039, 0.0000, 0.0000)),
    ])
}

/// Returns the default color scale for a given product.
///
/// This function selects the crate's built-in color scale for the product type.
///
/// | Product | Scale |
/// |---------|-------|
/// | Reflectivity | [`reflectivity_scale`] (-32 to 95 dBZ) |
/// | Velocity | Divergent Green-Red (-64 to +64 m/s) |
/// | SpectrumWidth | Sequential (0 to 30 m/s) |
/// | DifferentialReflectivity | Divergent (-2 to +6 dB) |
/// | DifferentialPhase | Sequential (0 to 360 deg) |
/// | CorrelationCoefficient | Sequential (0 to 1) |
/// | ClutterFilterPower | Divergent (-20 to +20 dB) |
/// | Any other product | Grayscale fallback over [`Product::value_range`] |
///
/// Products added to [`Product`] in future releases that do not yet have a
/// dedicated scale fall back to a neutral two-level grayscale scale.
///
/// # Example
///
/// ```
/// use nexrad_model::data::Product;
/// use nexrad_render::default_scale;
///
/// let scale = default_scale(Product::Reflectivity);
/// // Pass to render_sweep, render_cartesian or render_vertical,
/// // or build a ColorLookupTable from it.
/// ```
pub fn default_scale(product: Product) -> ColorScale {
    match product {
        Product::Reflectivity => reflectivity_scale(),
        Product::Velocity => velocity_scale(),
        Product::SpectrumWidth => spectrum_width_scale(),
        Product::DifferentialReflectivity => differential_reflectivity_scale(),
        Product::DifferentialPhase => differential_phase_scale(),
        Product::CorrelationCoefficient => correlation_coefficient_scale(),
        Product::ClutterFilterPower => clutter_filter_power_scale(),
        _ => fallback_scale(product),
    }
    .into()
}

/// Neutral fallback scale for products without a dedicated color scale.
///
/// Splits the product's [`Product::value_range`] at its midpoint into two gray
/// levels, so data remains visible without implying any meteorological meaning.
fn fallback_scale(product: Product) -> DiscreteColorScale {
    let (min, max) = product.value_range();
    DiscreteColorScale::from_levels(vec![
        ColorScaleLevel::new(min, Color::rgb(0.4, 0.4, 0.4)),
        ColorScaleLevel::new((min + max) / 2.0, Color::rgb(0.8, 0.8, 0.8)),
    ])
}
