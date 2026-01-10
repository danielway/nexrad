//! Tests for color scale functionality.

use nexrad_render::{get_nws_reflectivity_scale, ColorScaleLevel, DiscreteColorScale};
use piet::Color;

#[test]
fn test_discrete_color_scale_ordering() {
    // Levels should be sorted from highest to lowest internally
    let scale = DiscreteColorScale::new(vec![
        ColorScaleLevel::new(30.0, Color::GREEN),
        ColorScaleLevel::new(0.0, Color::BLACK),
        ColorScaleLevel::new(50.0, Color::RED),
    ]);

    // Value >= 50 should get red
    let color = scale.get_color(50.0);
    assert_eq!(color, Color::RED);

    // Value >= 30 but < 50 should get green
    let color = scale.get_color(35.0);
    assert_eq!(color, Color::GREEN);

    // Value >= 0 but < 30 should get black
    let color = scale.get_color(15.0);
    assert_eq!(color, Color::BLACK);
}

#[test]
fn test_color_scale_boundary_values() {
    let scale = DiscreteColorScale::new(vec![
        ColorScaleLevel::new(0.0, Color::BLACK),
        ColorScaleLevel::new(30.0, Color::GREEN),
        ColorScaleLevel::new(50.0, Color::RED),
    ]);

    // Exactly at threshold should get that threshold's color
    assert_eq!(scale.get_color(50.0), Color::RED);
    assert_eq!(scale.get_color(30.0), Color::GREEN);
    assert_eq!(scale.get_color(0.0), Color::BLACK);
}

#[test]
fn test_color_scale_negative_values() {
    let scale = DiscreteColorScale::new(vec![
        ColorScaleLevel::new(-10.0, Color::BLUE),
        ColorScaleLevel::new(0.0, Color::BLACK),
        ColorScaleLevel::new(30.0, Color::GREEN),
    ]);

    // Negative value above -10 threshold
    assert_eq!(scale.get_color(-5.0), Color::BLUE);

    // Very negative value (below all thresholds) should get lowest threshold color
    let color = scale.get_color(-20.0);
    assert_eq!(color, Color::BLUE);
}

#[test]
fn test_color_scale_high_values() {
    let scale = DiscreteColorScale::new(vec![
        ColorScaleLevel::new(0.0, Color::BLACK),
        ColorScaleLevel::new(50.0, Color::RED),
    ]);

    // Value above highest threshold should still get highest color
    assert_eq!(scale.get_color(100.0), Color::RED);
    assert_eq!(scale.get_color(75.0), Color::RED);
}

#[test]
fn test_nws_reflectivity_scale() {
    let scale = get_nws_reflectivity_scale();

    // Very low values (near threshold) should return a color
    let color = scale.get_color(0.0);
    // Black at 0 dBZ
    assert_eq!(color, Color::rgb(0.0, 0.0, 0.0));

    // Moderate reflectivity (green range, around 25 dBZ)
    let color = scale.get_color(25.0);
    // Should be in green range (RGB roughly 0.19, 0.80, 0.19)
    assert_eq!(color, Color::rgb(0.1961, 0.8039, 0.1961));

    // High reflectivity (red range, around 50 dBZ)
    let color = scale.get_color(50.0);
    assert_eq!(color, Color::rgb(1.0, 0.1882, 0.1882));

    // Extreme reflectivity (white, 75+ dBZ)
    let color = scale.get_color(80.0);
    assert_eq!(color, Color::rgb(1.0, 1.0, 1.0));
}

#[test]
fn test_color_scale_clone() {
    let scale = get_nws_reflectivity_scale();
    let cloned = scale.clone();

    // Both should produce same colors
    assert_eq!(scale.get_color(30.0), cloned.get_color(30.0));
    assert_eq!(scale.get_color(50.0), cloned.get_color(50.0));
}

#[test]
fn test_color_scale_debug() {
    let scale = get_nws_reflectivity_scale();
    let debug_str = format!("{:?}", scale);

    // Should have some content and not panic
    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("DiscreteColorScale"));
}
