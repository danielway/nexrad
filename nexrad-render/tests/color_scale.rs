//! Tests for color scale functionality.

use nexrad_model::data::Product;
use nexrad_render::{
    clutter_filter_power_scale, correlation_coefficient_scale, default_scale,
    differential_phase_scale, differential_reflectivity_scale, nws_reflectivity_scale,
    spectrum_width_scale, velocity_scale, Color, ColorLookupTable, ColorScale, ColorScaleLevel,
    ColorStop, ContinuousColorScale, DiscreteColorScale, Error,
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
    let color = scale.color(50.0);
    assert_eq!(color, Color::rgb(1.0, 0.0, 0.0));

    // Value >= 30 but < 50 should get green
    let color = scale.color(35.0);
    assert_eq!(color, Color::rgb(0.0, 1.0, 0.0));

    // Value >= 0 but < 30 should get black
    let color = scale.color(15.0);
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
    assert_eq!(scale.color(50.0), Color::rgb(1.0, 0.0, 0.0));
    assert_eq!(scale.color(30.0), Color::rgb(0.0, 1.0, 0.0));
    assert_eq!(scale.color(0.0), Color::BLACK);
}

#[test]
fn test_color_scale_negative_values() {
    let scale = DiscreteColorScale::new(vec![
        ColorScaleLevel::new(-10.0, Color::rgb(0.0, 0.0, 1.0)),
        ColorScaleLevel::new(0.0, Color::BLACK),
        ColorScaleLevel::new(30.0, Color::rgb(0.0, 1.0, 0.0)),
    ]);

    // Negative value above -10 threshold
    assert_eq!(scale.color(-5.0), Color::rgb(0.0, 0.0, 1.0));

    // Very negative value (below all thresholds) should get lowest threshold color
    let color = scale.color(-20.0);
    assert_eq!(color, Color::rgb(0.0, 0.0, 1.0));
}

#[test]
fn test_color_scale_high_values() {
    let scale = DiscreteColorScale::new(vec![
        ColorScaleLevel::new(0.0, Color::BLACK),
        ColorScaleLevel::new(50.0, Color::rgb(1.0, 0.0, 0.0)),
    ]);

    // Value above highest threshold should still get highest color
    assert_eq!(scale.color(100.0), Color::rgb(1.0, 0.0, 0.0));
    assert_eq!(scale.color(75.0), Color::rgb(1.0, 0.0, 0.0));
}

#[test]
fn test_nws_reflectivity_scale() {
    let scale = nws_reflectivity_scale();

    // Very low values (near threshold) should return a color
    let color = scale.color(0.0);
    // Black at 0 dBZ
    assert_eq!(color, Color::rgb(0.0, 0.0, 0.0));

    // Moderate reflectivity (green range, around 25 dBZ)
    let color = scale.color(25.0);
    // Should be in green range (RGB roughly 0.19, 0.80, 0.19)
    assert_eq!(color, Color::rgb(0.1961, 0.8039, 0.1961));

    // High reflectivity (red range, around 50 dBZ)
    let color = scale.color(50.0);
    assert_eq!(color, Color::rgb(1.0, 0.1882, 0.1882));

    // Extreme reflectivity (white, 75+ dBZ)
    let color = scale.color(80.0);
    assert_eq!(color, Color::rgb(1.0, 1.0, 1.0));
}

#[test]
fn test_color_scale_clone() {
    let scale = nws_reflectivity_scale();
    let cloned = scale.clone();

    // Both should produce same colors
    assert_eq!(scale.color(30.0), cloned.color(30.0));
    assert_eq!(scale.color(50.0), cloned.color(50.0));
}

#[test]
fn test_color_scale_debug() {
    let scale = nws_reflectivity_scale();
    let debug_str = format!("{:?}", scale);

    // Should have some content and not panic
    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("DiscreteColorScale"));
}

// Tests for velocity scale

#[test]
fn test_velocity_scale_negative() {
    let scale = velocity_scale();

    // Strong inbound velocity should be dark green
    let color = scale.color(-60.0);
    assert_eq!(color, Color::rgb(0.0, 0.3922, 0.0));

    // Moderate inbound should be green
    let color = scale.color(-40.0);
    assert_eq!(color, Color::rgb(0.0, 0.5451, 0.0));
}

#[test]
fn test_velocity_scale_zero() {
    let scale = velocity_scale();

    // Near-zero velocity should be gray
    let color = scale.color(0.0);
    assert_eq!(color, Color::rgb(0.6627, 0.6627, 0.6627));
}

#[test]
fn test_velocity_scale_positive() {
    let scale = velocity_scale();

    // Strong outbound velocity (>= 64) should be dark red
    let color = scale.color(64.0);
    assert_eq!(color, Color::rgb(0.5451, 0.0, 0.0));

    // Moderate-strong outbound (48-64 range) should be red
    let color = scale.color(50.0);
    assert_eq!(color, Color::rgb(0.8039, 0.0, 0.0));

    // Moderate outbound (32-48 range) should be light red
    let color = scale.color(40.0);
    assert_eq!(color, Color::rgb(1.0, 0.4118, 0.4118));
}

// Tests for spectrum width scale

#[test]
fn test_spectrum_width_scale() {
    let scale = spectrum_width_scale();

    // Low turbulence should be gray
    let color = scale.color(2.0);
    assert_eq!(color, Color::rgb(0.502, 0.502, 0.502));

    // Moderate turbulence should be green
    let color = scale.color(14.0);
    assert_eq!(color, Color::rgb(0.0, 0.8039, 0.0));

    // High turbulence should be red
    let color = scale.color(28.0);
    assert_eq!(color, Color::rgb(1.0, 0.0, 0.0));
}

// Tests for differential reflectivity scale

#[test]
fn test_differential_reflectivity_scale() {
    let scale = differential_reflectivity_scale();

    // Negative ZDR (vertically oriented) should be purple/blue
    let color = scale.color(-1.5);
    assert_eq!(color, Color::rgb(0.502, 0.0, 0.502));

    // Near-zero ZDR should be gray
    let color = scale.color(0.25);
    assert_eq!(color, Color::rgb(0.6627, 0.6627, 0.6627));

    // Positive ZDR (oblate drops) should be in yellow/orange/red range
    let color = scale.color(3.0);
    assert_eq!(color, Color::rgb(1.0, 0.6471, 0.0));
}

// Tests for correlation coefficient scale

#[test]
fn test_correlation_coefficient_scale() {
    let scale = correlation_coefficient_scale();

    // Low CC (debris/non-met) should be dark
    let color = scale.color(0.3);
    assert_eq!(color, Color::rgb(0.3922, 0.0, 0.5882));

    // High CC (pure precipitation) should be light
    let color = scale.color(0.99);
    assert_eq!(color, Color::rgb(0.902, 0.902, 0.902));
}

// Tests for differential phase scale

#[test]
fn test_differential_phase_scale() {
    let scale = differential_phase_scale();

    // Low phase should be purple
    let color = scale.color(20.0);
    assert_eq!(color, Color::rgb(0.502, 0.0, 0.502));

    // Mid phase should be green
    let color = scale.color(150.0);
    assert_eq!(color, Color::rgb(0.0, 0.8039, 0.0));

    // High phase should be red/magenta
    let color = scale.color(300.0);
    assert_eq!(color, Color::rgb(1.0, 0.0, 0.0));
}

// Tests for clutter filter power scale

#[test]
fn test_clutter_filter_power_scale() {
    let scale = clutter_filter_power_scale();

    // Negative CFP should be blue
    let color = scale.color(-15.0);
    assert_eq!(color, Color::rgb(0.0, 0.0, 0.5451));

    // Near-zero CFP should be gray
    let color = scale.color(0.0);
    assert_eq!(color, Color::rgb(0.6627, 0.6627, 0.6627));

    // Positive CFP should be red
    let color = scale.color(12.0);
    assert_eq!(color, Color::rgb(1.0, 0.4118, 0.4118));
}

// Tests for default_scale

#[test]
fn test_default_scale_reflectivity() {
    let scale = default_scale(Product::Reflectivity);
    let nws_scale = nws_reflectivity_scale();

    // Should return the same colors as NWS reflectivity scale
    assert_eq!(scale.color(30.0), nws_scale.color(30.0));
    assert_eq!(scale.color(50.0), nws_scale.color(50.0));
}

#[test]
fn test_default_scale_velocity() {
    let scale = default_scale(Product::Velocity);
    let velocity_scale = velocity_scale();

    // Should return the same colors as velocity scale
    assert_eq!(scale.color(-32.0), velocity_scale.color(-32.0));
    assert_eq!(scale.color(32.0), velocity_scale.color(32.0));
}

#[test]
fn test_default_scale_all_products() {
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
        let scale = default_scale(product);
        // Just verify we can get colors without panicking
        let _ = scale.color(0.0);
        let _ = scale.color(50.0);
    }
}

// Tests for Color

#[test]
fn test_color_to_rgba8_rounds_to_nearest() {
    // 0.5 * 255 = 127.5 rounds up; 0.2 * 255 = 51.000..01 stays at 51
    assert_eq!(
        Color::rgba(0.5, 0.2, 0.999, 0.001).to_rgba8(),
        [128, 51, 255, 0]
    );
    // 0.6627 * 255 = 168.99 rounds to 169 instead of truncating to 168
    assert_eq!(Color::rgb(0.6627, 0.0, 1.0).to_rgba8(), [169, 0, 255, 255]);
}

#[test]
fn test_color_to_rgba8_clamps() {
    assert_eq!(
        Color::rgba(-0.5, 1.5, 2.0, -1.0).to_rgba8(),
        [0, 255, 255, 0]
    );
    assert_eq!(
        Color::rgba(f64::NAN, 0.0, 0.0, 1.0).to_rgba8(),
        [0, 0, 0, 255]
    );
}

#[test]
fn test_color_from_rgba8_round_trips() {
    for byte in 0..=255u8 {
        let rgba = [byte, 255 - byte, byte / 2, 255];
        assert_eq!(Color::from_rgba8(rgba).to_rgba8(), rgba);
        assert_eq!(<[u8; 4]>::from(Color::from(rgba)), rgba);
    }
    assert_eq!(
        Color::from_rgba8([255, 0, 0, 255]),
        Color::rgb(1.0, 0.0, 0.0)
    );
}

#[test]
fn test_color_default_is_transparent() {
    assert_eq!(Color::default(), Color::TRANSPARENT);
}

#[test]
fn test_color_scale_level_fields() {
    let scale = DiscreteColorScale::new(vec![
        ColorScaleLevel::new(0.0, Color::BLACK),
        ColorScaleLevel::new(30.0, Color::WHITE),
    ]);
    let legend: Vec<(f32, Color)> = scale
        .levels()
        .iter()
        .map(|level| (level.value, level.color))
        .collect();
    assert_eq!(legend, vec![(30.0, Color::WHITE), (0.0, Color::BLACK)]);
}

// Tests for ColorLookupTable

#[test]
fn test_lookup_table_matches_scale_at_range_ends() {
    let scale = default_scale(Product::Reflectivity);
    let lut = ColorLookupTable::new(&scale, -32.0, 95.0, 256).unwrap();

    assert_eq!(lut.color(-32.0), scale.color(-32.0).to_rgba8());
    assert_eq!(lut.color(95.0), scale.color(95.0).to_rgba8());
}

#[test]
fn test_lookup_table_accessors() {
    let scale = default_scale(Product::Reflectivity);

    for size in [2, 17, 256] {
        let lut = ColorLookupTable::new(&scale, 0.0, 70.0, size).unwrap();
        assert_eq!(lut.len(), size);
        assert!(!lut.is_empty());
        assert_eq!(lut.entries().len(), lut.len());
        assert_eq!(lut.min_value(), 0.0);
        assert_eq!(lut.max_value(), 70.0);
        assert_eq!(lut.entries()[0], scale.color(0.0).to_rgba8());
        assert_eq!(lut.entries()[size - 1], scale.color(70.0).to_rgba8());
        assert_eq!(lut.color(0.0), lut.entries()[0]);
        assert_eq!(lut.color(70.0), lut.entries()[size - 1]);
    }
}

#[test]
fn test_lookup_table_rejects_invalid_ranges() {
    let scale = default_scale(Product::Reflectivity);

    for (min, max) in [
        (10.0, 10.0),
        (20.0, 10.0),
        (f32::NAN, 10.0),
        (0.0, f32::INFINITY),
        (f32::NEG_INFINITY, 0.0),
    ] {
        assert!(
            matches!(
                ColorLookupTable::new(&scale, min, max, 256),
                Err(Error::InvalidRange { .. })
            ),
            "{min}..{max} should be rejected"
        );
    }
}

#[test]
fn test_lookup_table_rejects_tiny_sizes() {
    let scale = default_scale(Product::Reflectivity);

    for size in [0, 1] {
        assert!(matches!(
            ColorLookupTable::new(&scale, 0.0, 70.0, size),
            Err(Error::InvalidTableSize { size: s }) if s == size
        ));
    }
}

/// Every built-in discrete scale with the product range it is used over.
fn builtin_discrete_scales() -> Vec<(DiscreteColorScale, Product)> {
    vec![
        (nws_reflectivity_scale(), Product::Reflectivity),
        (velocity_scale(), Product::Velocity),
        (spectrum_width_scale(), Product::SpectrumWidth),
        (
            differential_reflectivity_scale(),
            Product::DifferentialReflectivity,
        ),
        (
            correlation_coefficient_scale(),
            Product::CorrelationCoefficient,
        ),
        (differential_phase_scale(), Product::DifferentialPhase),
        (clutter_filter_power_scale(), Product::ClutterFilterPower),
    ]
}

#[test]
fn test_lookup_table_threshold_values_get_their_own_color() {
    for (discrete, product) in builtin_discrete_scales() {
        let (min, max) = product.value_range();
        let scale = ColorScale::from(discrete.clone());

        for size in [256, 16, 2] {
            let lut = ColorLookupTable::new(&scale, min, max, size).unwrap();
            for level in discrete.levels() {
                assert_eq!(
                    lut.color(level.value),
                    level.color.to_rgba8(),
                    "{product:?} level {} with {size} entries",
                    level.value
                );
            }
        }
    }
}

#[test]
fn test_lookup_table_matches_discrete_scale_between_thresholds() {
    for (discrete, product) in builtin_discrete_scales() {
        let (min, max) = product.value_range();
        let scale = ColorScale::from(discrete);
        let lut = ColorLookupTable::new(&scale, min, max, 16).unwrap();

        for step in 0..=1000 {
            let value = min + (max - min) * step as f32 / 1000.0;
            assert_eq!(
                lut.color(value),
                scale.color(value).to_rgba8(),
                "{product:?} value {value}"
            );
        }
        // Out-of-range values take the color at the nearer end; NaN the minimum's.
        assert_eq!(lut.color(max + 1000.0), scale.color(max).to_rgba8());
        assert_eq!(lut.color(min - 1000.0), scale.color(min).to_rgba8());
        assert_eq!(lut.color(f32::NAN), scale.color(min).to_rgba8());
    }

    // Just below a threshold keeps the lower level's color.
    let scale = ColorScale::from(nws_reflectivity_scale());
    let lut = ColorLookupTable::new(&scale, -32.0, 95.0, 16).unwrap();
    let below = 20.0f32.next_down();
    assert_eq!(lut.color(below), scale.color(15.0).to_rgba8());
    assert_ne!(lut.color(below), lut.color(20.0));
}

#[test]
fn test_lookup_table_interpolates_continuous_scales() {
    let scale = ColorScale::from(ContinuousColorScale::new(vec![
        ColorStop::new(0.0, Color::BLACK),
        ColorStop::new(100.0, Color::WHITE),
    ]));
    let lut = ColorLookupTable::new(&scale, 0.0, 100.0, 256).unwrap();

    assert_eq!(lut.color(0.0), [0, 0, 0, 255]);
    assert_eq!(lut.color(100.0), [255, 255, 255, 255]);
    for value in [12.5f32, 25.0, 50.0, 73.0] {
        let expected = scale.color(value).to_rgba8()[0];
        let actual = lut.color(value)[0];
        assert!(
            actual.abs_diff(expected) <= 1,
            "value {value}: {actual} vs {expected}"
        );
    }
}
