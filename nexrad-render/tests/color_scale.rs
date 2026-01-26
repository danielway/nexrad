//! Tests for color scale functionality.

use nexrad_render::{
    get_clutter_filter_power_scale, get_correlation_coefficient_scale, get_default_scale,
    get_differential_phase_scale, get_differential_reflectivity_scale,
    get_nws_reflectivity_scale, get_spectrum_width_scale, get_velocity_scale, Color,
    ColorScaleLevel, DiscreteColorScale, Product,
};

#[test]
fn test_discrete_color_scale_ordering() {
    // Levels should be sorted from highest to lowest internally
    let scale = DiscreteColorScale::new(vec![
        ColorScaleLevel::new(30.0, Color::rgb(0.0, 1.0, 0.0)),
        ColorScaleLevel::new(0.0, Color::BLACK),
        ColorScaleLevel::new(50.0, Color::rgb(1.0, 0.0, 0.0)),
    ]);

    // Value >= 50 should get red
    let color = scale.get_color(50.0);
    assert_eq!(color, Color::rgb(1.0, 0.0, 0.0));

    // Value >= 30 but < 50 should get green
    let color = scale.get_color(35.0);
    assert_eq!(color, Color::rgb(0.0, 1.0, 0.0));

    // Value >= 0 but < 30 should get black
    let color = scale.get_color(15.0);
    assert_eq!(color, Color::BLACK);
}

#[test]
fn test_color_scale_boundary_values() {
    let scale = DiscreteColorScale::new(vec![
        ColorScaleLevel::new(0.0, Color::BLACK),
        ColorScaleLevel::new(30.0, Color::rgb(0.0, 1.0, 0.0)),
        ColorScaleLevel::new(50.0, Color::rgb(1.0, 0.0, 0.0)),
    ]);

    // Exactly at threshold should get that threshold's color
    assert_eq!(scale.get_color(50.0), Color::rgb(1.0, 0.0, 0.0));
    assert_eq!(scale.get_color(30.0), Color::rgb(0.0, 1.0, 0.0));
    assert_eq!(scale.get_color(0.0), Color::BLACK);
}

#[test]
fn test_color_scale_negative_values() {
    let scale = DiscreteColorScale::new(vec![
        ColorScaleLevel::new(-10.0, Color::rgb(0.0, 0.0, 1.0)),
        ColorScaleLevel::new(0.0, Color::BLACK),
        ColorScaleLevel::new(30.0, Color::rgb(0.0, 1.0, 0.0)),
    ]);

    // Negative value above -10 threshold
    assert_eq!(scale.get_color(-5.0), Color::rgb(0.0, 0.0, 1.0));

    // Very negative value (below all thresholds) should get lowest threshold color
    let color = scale.get_color(-20.0);
    assert_eq!(color, Color::rgb(0.0, 0.0, 1.0));
}

#[test]
fn test_color_scale_high_values() {
    let scale = DiscreteColorScale::new(vec![
        ColorScaleLevel::new(0.0, Color::BLACK),
        ColorScaleLevel::new(50.0, Color::rgb(1.0, 0.0, 0.0)),
    ]);

    // Value above highest threshold should still get highest color
    assert_eq!(scale.get_color(100.0), Color::rgb(1.0, 0.0, 0.0));
    assert_eq!(scale.get_color(75.0), Color::rgb(1.0, 0.0, 0.0));
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

// Tests for velocity scale

#[test]
fn test_velocity_scale_negative() {
    let scale = get_velocity_scale();

    // Strong inbound velocity should be dark green
    let color = scale.get_color(-60.0);
    assert_eq!(color, Color::rgb(0.0, 0.3922, 0.0));

    // Moderate inbound should be green
    let color = scale.get_color(-40.0);
    assert_eq!(color, Color::rgb(0.0, 0.5451, 0.0));
}

#[test]
fn test_velocity_scale_zero() {
    let scale = get_velocity_scale();

    // Near-zero velocity should be gray
    let color = scale.get_color(0.0);
    assert_eq!(color, Color::rgb(0.6627, 0.6627, 0.6627));
}

#[test]
fn test_velocity_scale_positive() {
    let scale = get_velocity_scale();

    // Strong outbound velocity (>= 64) should be dark red
    let color = scale.get_color(64.0);
    assert_eq!(color, Color::rgb(0.5451, 0.0, 0.0));

    // Moderate-strong outbound (48-64 range) should be red
    let color = scale.get_color(50.0);
    assert_eq!(color, Color::rgb(0.8039, 0.0, 0.0));

    // Moderate outbound (32-48 range) should be light red
    let color = scale.get_color(40.0);
    assert_eq!(color, Color::rgb(1.0, 0.4118, 0.4118));
}

// Tests for spectrum width scale

#[test]
fn test_spectrum_width_scale() {
    let scale = get_spectrum_width_scale();

    // Low turbulence should be gray
    let color = scale.get_color(2.0);
    assert_eq!(color, Color::rgb(0.502, 0.502, 0.502));

    // Moderate turbulence should be green
    let color = scale.get_color(14.0);
    assert_eq!(color, Color::rgb(0.0, 0.8039, 0.0));

    // High turbulence should be red
    let color = scale.get_color(28.0);
    assert_eq!(color, Color::rgb(1.0, 0.0, 0.0));
}

// Tests for differential reflectivity scale

#[test]
fn test_differential_reflectivity_scale() {
    let scale = get_differential_reflectivity_scale();

    // Negative ZDR (vertically oriented) should be purple/blue
    let color = scale.get_color(-1.5);
    assert_eq!(color, Color::rgb(0.502, 0.0, 0.502));

    // Near-zero ZDR should be gray
    let color = scale.get_color(0.25);
    assert_eq!(color, Color::rgb(0.6627, 0.6627, 0.6627));

    // Positive ZDR (oblate drops) should be in yellow/orange/red range
    let color = scale.get_color(3.0);
    assert_eq!(color, Color::rgb(1.0, 0.6471, 0.0));
}

// Tests for correlation coefficient scale

#[test]
fn test_correlation_coefficient_scale() {
    let scale = get_correlation_coefficient_scale();

    // Low CC (debris/non-met) should be dark
    let color = scale.get_color(0.3);
    assert_eq!(color, Color::rgb(0.3922, 0.0, 0.5882));

    // High CC (pure precipitation) should be light
    let color = scale.get_color(0.99);
    assert_eq!(color, Color::rgb(0.902, 0.902, 0.902));
}

// Tests for differential phase scale

#[test]
fn test_differential_phase_scale() {
    let scale = get_differential_phase_scale();

    // Low phase should be purple
    let color = scale.get_color(20.0);
    assert_eq!(color, Color::rgb(0.502, 0.0, 0.502));

    // Mid phase should be green
    let color = scale.get_color(150.0);
    assert_eq!(color, Color::rgb(0.0, 0.8039, 0.0));

    // High phase should be red/magenta
    let color = scale.get_color(300.0);
    assert_eq!(color, Color::rgb(1.0, 0.0, 0.0));
}

// Tests for clutter filter power scale

#[test]
fn test_clutter_filter_power_scale() {
    let scale = get_clutter_filter_power_scale();

    // Negative CFP should be blue
    let color = scale.get_color(-15.0);
    assert_eq!(color, Color::rgb(0.0, 0.0, 0.5451));

    // Near-zero CFP should be gray
    let color = scale.get_color(0.0);
    assert_eq!(color, Color::rgb(0.6627, 0.6627, 0.6627));

    // Positive CFP should be red
    let color = scale.get_color(12.0);
    assert_eq!(color, Color::rgb(1.0, 0.4118, 0.4118));
}

// Tests for get_default_scale

#[test]
fn test_get_default_scale_reflectivity() {
    let scale = get_default_scale(Product::Reflectivity);
    let nws_scale = get_nws_reflectivity_scale();

    // Should return the same colors as NWS reflectivity scale
    assert_eq!(scale.get_color(30.0), nws_scale.get_color(30.0));
    assert_eq!(scale.get_color(50.0), nws_scale.get_color(50.0));
}

#[test]
fn test_get_default_scale_velocity() {
    let scale = get_default_scale(Product::Velocity);
    let velocity_scale = get_velocity_scale();

    // Should return the same colors as velocity scale
    assert_eq!(scale.get_color(-32.0), velocity_scale.get_color(-32.0));
    assert_eq!(scale.get_color(32.0), velocity_scale.get_color(32.0));
}

#[test]
fn test_get_default_scale_all_products() {
    // Verify all products return a valid scale
    let products = [
        Product::Reflectivity,
        Product::Velocity,
        Product::SpectrumWidth,
        Product::DifferentialReflectivity,
        Product::DifferentialPhase,
        Product::CorrelationCoefficient,
        Product::ClutterFilterPower,
    ];

    for product in products {
        let scale = get_default_scale(product);
        // Just verify we can get colors without panicking
        let _ = scale.get_color(0.0);
        let _ = scale.get_color(50.0);
    }
}
