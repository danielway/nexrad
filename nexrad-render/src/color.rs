//! Color scales for radar data visualization.
//!
//! This module provides types for mapping radar moment values to colors.
//! The primary type is [`DiscreteColorScale`], which maps value ranges to
//! specific colors based on threshold levels. For rendering, the scale is
//! converted to a [`ColorLookupTable`] which provides O(1) color lookups.

/// An RGBA color with components in the range 0.0 to 1.0.
///
/// This type is used for defining color scales. Colors are converted to
/// 8-bit RGBA values during rendering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    /// Red component (0.0 to 1.0)
    pub r: f64,
    /// Green component (0.0 to 1.0)
    pub g: f64,
    /// Blue component (0.0 to 1.0)
    pub b: f64,
    /// Alpha component (0.0 to 1.0)
    pub a: f64,
}

impl Color {
    /// Creates a new color from RGB components (alpha defaults to 1.0).
    ///
    /// Components should be in the range 0.0 to 1.0.
    pub const fn rgb(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Creates a new color from RGBA components.
    ///
    /// Components should be in the range 0.0 to 1.0.
    pub const fn rgba(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }

    /// Converts the color to 8-bit RGBA values.
    pub fn to_rgba8(&self) -> [u8; 4] {
        [
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            (self.a * 255.0) as u8,
        ]
    }

    /// Black color.
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);

    /// White color.
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);

    /// Transparent (fully transparent black).
    pub const TRANSPARENT: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);
}

/// A single level in a discrete color scale.
///
/// Represents a threshold value and its associated color. Values at or above
/// this threshold (but below the next higher threshold) will be rendered with
/// this color.
#[derive(Debug, Clone)]
pub struct ColorScaleLevel {
    value: f32,
    color: Color,
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
/// ]);
///
/// // Values >= 50 return red, >= 30 return green, >= 0 return black
/// ```
#[derive(Debug, Clone)]
pub struct DiscreteColorScale {
    levels: Vec<ColorScaleLevel>,
}

impl DiscreteColorScale {
    /// Creates a new discrete color scale from the given levels.
    ///
    /// Levels are automatically sorted from highest to lowest threshold.
    pub fn new(mut levels: Vec<ColorScaleLevel>) -> Self {
        levels.sort_by(|a, b| b.value.total_cmp(&a.value));
        Self { levels }
    }

    /// Returns the color for the given value.
    ///
    /// Finds the highest threshold that the value exceeds and returns its color.
    /// If the value is below all thresholds, returns the color of the lowest threshold.
    pub fn get_color(&self, value: f32) -> Color {
        let mut color = Color::BLACK;

        for level in &self.levels {
            if value >= level.value {
                return level.color;
            }

            color = level.color;
        }

        color
    }

    /// Returns the levels in this color scale (sorted highest to lowest).
    pub fn levels(&self) -> &[ColorScaleLevel] {
        &self.levels
    }
}

/// A pre-computed lookup table for O(1) color lookups.
///
/// This table maps a range of values to RGBA colors using a fixed-size array.
/// It is created from a [`DiscreteColorScale`] and provides fast color lookups
/// during rendering.
///
/// # Example
///
/// ```
/// use nexrad_render::{ColorLookupTable, get_nws_reflectivity_scale};
///
/// let scale = get_nws_reflectivity_scale();
/// let lut = ColorLookupTable::from_scale(&scale, -32.0, 95.0, 256);
///
/// // O(1) lookup returning [R, G, B, A] bytes
/// let color = lut.get_color(45.0);
/// ```
#[derive(Debug, Clone)]
pub struct ColorLookupTable {
    /// RGBA color values indexed by quantized input value.
    table: Vec<[u8; 4]>,
    /// Minimum value in the mapped range.
    min_value: f32,
    /// Value range (max - min).
    range: f32,
}

impl ColorLookupTable {
    /// Creates a lookup table from a discrete color scale.
    ///
    /// # Arguments
    ///
    /// * `scale` - The color scale to sample from
    /// * `min_value` - The minimum value to map
    /// * `max_value` - The maximum value to map
    /// * `size` - The number of entries in the lookup table (256 recommended)
    ///
    /// Values outside the min/max range will be clamped to the nearest entry.
    pub fn from_scale(
        scale: &DiscreteColorScale,
        min_value: f32,
        max_value: f32,
        size: usize,
    ) -> Self {
        let range = max_value - min_value;
        let mut table = Vec::with_capacity(size);

        for i in 0..size {
            let value = min_value + (i as f32 / (size - 1) as f32) * range;
            let color = scale.get_color(value);
            table.push(color.to_rgba8());
        }

        Self {
            table,
            min_value,
            range,
        }
    }

    /// Returns the RGBA color for the given value.
    ///
    /// This is an O(1) operation using direct array indexing.
    #[inline]
    pub fn get_color(&self, value: f32) -> [u8; 4] {
        let normalized = (value - self.min_value) / self.range;
        let index = (normalized * (self.table.len() - 1) as f32) as usize;
        let index = index.min(self.table.len() - 1);
        self.table[index]
    }
}

/// Returns the standard NWS (National Weather Service) reflectivity color scale.
///
/// This scale uses colors commonly seen in weather radar displays, ranging
/// from cyan/blue for light precipitation to magenta/white for extreme values.
///
/// | dBZ Range | Color | Meaning |
/// |-----------|-------|---------|
/// | 0-5 | Black | Below detection threshold |
/// | 5-20 | Cyan/Blue | Light precipitation |
/// | 20-35 | Green | Light to moderate precipitation |
/// | 35-50 | Yellow/Orange | Moderate to heavy precipitation |
/// | 50-65 | Red/Magenta | Heavy precipitation, possible hail |
/// | 65+ | Purple/White | Extreme precipitation, likely hail |
pub fn get_nws_reflectivity_scale() -> DiscreteColorScale {
    DiscreteColorScale::new(vec![
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
/// This divergent scale uses green for motion toward the radar (negative values)
/// and red for motion away from the radar (positive values), with gray near zero.
/// Range: -64 to +64 m/s (standard precipitation mode Doppler velocity).
///
/// | Velocity (m/s) | Color | Meaning |
/// |----------------|-------|---------|
/// | -64 to -48 | Dark Green | Strong inbound |
/// | -48 to -32 | Green | Moderate inbound |
/// | -32 to -16 | Light Green | Light inbound |
/// | -16 to -4 | Pale Green | Very light inbound |
/// | -4 to +4 | Gray | Near zero / RF |
/// | +4 to +16 | Pale Red | Very light outbound |
/// | +16 to +32 | Light Red/Pink | Light outbound |
/// | +32 to +48 | Red | Moderate outbound |
/// | +48 to +64 | Dark Red | Strong outbound |
pub fn get_velocity_scale() -> DiscreteColorScale {
    DiscreteColorScale::new(vec![
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
/// This sequential scale ranges from cool colors (low turbulence) to warm colors
/// (high turbulence). Range: 0 to 30 m/s.
///
/// | Width (m/s) | Color | Meaning |
/// |-------------|-------|---------|
/// | 0-4 | Gray | Very low turbulence |
/// | 4-8 | Blue | Low turbulence |
/// | 8-12 | Cyan | Light turbulence |
/// | 12-16 | Green | Moderate turbulence |
/// | 16-20 | Yellow | Moderate-high turbulence |
/// | 20-25 | Orange | High turbulence |
/// | 25-30 | Red | Very high turbulence |
pub fn get_spectrum_width_scale() -> DiscreteColorScale {
    DiscreteColorScale::new(vec![
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
/// This divergent scale shows negative values (vertically-oriented particles) in
/// blue/purple, near-zero in gray, and positive values (horizontally-oriented
/// particles like large raindrops) in yellow/orange/red.
/// Range: -2 to +6 dB.
///
/// | ZDR (dB) | Color | Meaning |
/// |----------|-------|---------|
/// | -2 to -1 | Purple | Vertically oriented |
/// | -1 to 0 | Blue | Slightly vertical |
/// | 0 to 0.5 | Gray | Spherical particles |
/// | 0.5 to 1.5 | Light Green | Slightly oblate |
/// | 1.5 to 2.5 | Yellow | Oblate drops |
/// | 2.5 to 4 | Orange | Large oblate drops |
/// | 4 to 6 | Red | Very large drops/hail |
pub fn get_differential_reflectivity_scale() -> DiscreteColorScale {
    DiscreteColorScale::new(vec![
        // Negative (vertically oriented)
        ColorScaleLevel::new(-2.0, Color::rgb(0.5020, 0.0000, 0.5020)),
        ColorScaleLevel::new(-1.0, Color::rgb(0.0000, 0.0000, 0.8039)),
        // Near zero (spherical)
        ColorScaleLevel::new(0.0, Color::rgb(0.6627, 0.6627, 0.6627)),
        // Positive (horizontally oriented / oblate)
        ColorScaleLevel::new(0.5, Color::rgb(0.5647, 0.9333, 0.5647)),
        ColorScaleLevel::new(1.5, Color::rgb(0.9333, 0.9333, 0.0000)),
        ColorScaleLevel::new(2.5, Color::rgb(1.0000, 0.6471, 0.0000)),
        ColorScaleLevel::new(4.0, Color::rgb(1.0000, 0.0000, 0.0000)),
    ])
}

/// Returns a color scale for correlation coefficient (CC/RhoHV) data.
///
/// This sequential scale emphasizes high values (0.9-1.0) which indicate
/// meteorological targets. Lower values may indicate non-meteorological
/// echoes, mixed precipitation, or tornadic debris.
/// Range: 0.0 to 1.0.
///
/// | CC | Color | Meaning |
/// |----|-------|---------|
/// | 0.0-0.7 | Purple/Blue | Non-met or debris |
/// | 0.7-0.85 | Cyan/Teal | Mixed phase/melting |
/// | 0.85-0.92 | Light Green | Possible hail/graupel |
/// | 0.92-0.96 | Green | Rain/snow mix |
/// | 0.96-0.98 | Yellow | Pure rain or snow |
/// | 0.98-1.0 | White/Light Gray | Uniform precipitation |
pub fn get_correlation_coefficient_scale() -> DiscreteColorScale {
    DiscreteColorScale::new(vec![
        // Low CC - non-meteorological or debris
        ColorScaleLevel::new(0.0, Color::rgb(0.0000, 0.0000, 0.0000)),
        ColorScaleLevel::new(0.2, Color::rgb(0.3922, 0.0000, 0.5882)),
        ColorScaleLevel::new(0.5, Color::rgb(0.0000, 0.0000, 0.8039)),
        ColorScaleLevel::new(0.7, Color::rgb(0.0000, 0.5451, 0.5451)),
        // Medium CC - mixed precipitation
        ColorScaleLevel::new(0.85, Color::rgb(0.0000, 0.8039, 0.4000)),
        ColorScaleLevel::new(0.92, Color::rgb(0.0000, 0.8039, 0.0000)),
        // High CC - pure meteorological
        ColorScaleLevel::new(0.96, Color::rgb(0.9333, 0.9333, 0.0000)),
        ColorScaleLevel::new(0.98, Color::rgb(0.9020, 0.9020, 0.9020)),
    ])
}

/// Returns a color scale for differential phase (PhiDP) data.
///
/// This sequential scale covers the full 0-360 degree range. Differential
/// phase increases with propagation through precipitation.
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
pub fn get_differential_phase_scale() -> DiscreteColorScale {
    DiscreteColorScale::new(vec![
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

/// Returns a color scale for specific differential phase (KDP) data.
///
/// This sequential scale shows the rate of differential phase change,
/// which correlates with rainfall rate. Higher KDP indicates heavier rain.
/// Range: 0 to 10 degrees/km.
///
/// | KDP (deg/km) | Color | Meaning |
/// |--------------|-------|---------|
/// | 0-0.5 | Gray | Very light/no rain |
/// | 0.5-1.0 | Light Blue | Light rain |
/// | 1.0-2.0 | Blue | Light-moderate rain |
/// | 2.0-3.0 | Green | Moderate rain |
/// | 3.0-4.5 | Yellow | Moderate-heavy rain |
/// | 4.5-6.0 | Orange | Heavy rain |
/// | 6.0-10.0 | Red | Very heavy rain |
pub fn get_specific_diff_phase_scale() -> DiscreteColorScale {
    DiscreteColorScale::new(vec![
        ColorScaleLevel::new(0.0, Color::rgb(0.6627, 0.6627, 0.6627)),
        ColorScaleLevel::new(0.5, Color::rgb(0.6784, 0.8471, 0.9020)),
        ColorScaleLevel::new(1.0, Color::rgb(0.0000, 0.0000, 0.8039)),
        ColorScaleLevel::new(2.0, Color::rgb(0.0000, 0.8039, 0.0000)),
        ColorScaleLevel::new(3.0, Color::rgb(0.9333, 0.9333, 0.0000)),
        ColorScaleLevel::new(4.5, Color::rgb(1.0000, 0.6471, 0.0000)),
        ColorScaleLevel::new(6.0, Color::rgb(1.0000, 0.0000, 0.0000)),
    ])
}
