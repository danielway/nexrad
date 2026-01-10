//! Color scales for radar data visualization.
//!
//! This module provides types for mapping radar moment values to colors.
//! The primary type is [`DiscreteColorScale`], which maps value ranges to
//! specific colors based on threshold levels.

use piet::Color;

/// A single level in a discrete color scale.
///
/// Represents a threshold value and its associated color. Values at or above
/// this threshold (but below the next higher threshold) will be rendered with
/// this color.
#[derive(Debug, Clone)]
pub struct ColorScaleLevel {
    value_dbz: f32,
    color: Color,
}

impl ColorScaleLevel {
    /// Creates a new color scale level.
    ///
    /// # Arguments
    ///
    /// * `value_dbz` - The threshold value (typically in dBZ for reflectivity)
    /// * `color` - The color to use for values at or above this threshold
    pub fn new(value_dbz: f32, color: Color) -> Self {
        Self { value_dbz, color }
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
/// use nexrad_render::{ColorScaleLevel, DiscreteColorScale};
/// use piet::Color;
///
/// let scale = DiscreteColorScale::new(vec![
///     ColorScaleLevel::new(0.0, Color::BLACK),
///     ColorScaleLevel::new(30.0, Color::GREEN),
///     ColorScaleLevel::new(50.0, Color::RED),
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
        levels.sort_by(|a, b| b.value_dbz.total_cmp(&a.value_dbz));
        Self { levels }
    }

    /// Returns the color for the given value.
    ///
    /// Finds the highest threshold that the value exceeds and returns its color.
    /// If the value is below all thresholds, returns the color of the lowest threshold.
    pub fn get_color(&self, value_dbz: f32) -> Color {
        let mut color = Color::BLACK;

        for level in &self.levels {
            if value_dbz >= level.value_dbz {
                return level.color;
            }

            color = level.color;
        }

        color
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
