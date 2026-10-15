//! Tests for RenderOptions.

use nexrad_model::data::GateStatus;
use nexrad_model::geo::{GeoExtent, GeoPoint};
use nexrad_render::{Color, Interpolation, RenderOptions};

#[test]
fn test_render_options_new() {
    let options = RenderOptions::new(800, 600);

    assert_eq!(options.size(), (800, 600));
    // Default background should be black (opaque)
    assert_eq!(options.background(), Some(Color::BLACK));
    assert_eq!(options.interpolation(), Interpolation::Nearest);
    assert_eq!(options.value_range(), None);
    assert!(options.extent().is_none());
    assert!(options.coord_system().is_none());
    for status in [
        GateStatus::Valid,
        GateStatus::BelowThreshold,
        GateStatus::RangeFolded,
        GateStatus::NoData,
        GateStatus::Filtered,
    ] {
        assert_eq!(options.status_color(status), None);
    }
}

#[test]
fn test_render_options_transparent_background() {
    let options = RenderOptions::new(800, 600).with_background(Color::TRANSPARENT);

    assert_eq!(options.size(), (800, 600));
    assert_eq!(options.background(), None);
}

#[test]
fn test_render_options_with_background() {
    let options = RenderOptions::new(800, 600).with_background(Color::WHITE);

    assert_eq!(options.size(), (800, 600));
    assert_eq!(options.background(), Some(Color::WHITE));
}

#[test]
fn test_render_options_chain() {
    let grey = Color::from_rgba8([128, 128, 128, 255]);
    let options = RenderOptions::new(1000, 1000)
        .with_background(Color::TRANSPARENT)
        .with_background(grey);

    assert_eq!(options.size(), (1000, 1000));
    // Last call wins
    assert_eq!(options.background(), Some(grey));
}

#[test]
fn test_render_options_clone_and_eq() {
    let options = RenderOptions::new(800, 600)
        .with_background(Color::TRANSPARENT)
        .with_status_color(GateStatus::RangeFolded, Color::WHITE);
    let cloned = options.clone();

    assert_eq!(options, cloned);
    assert_ne!(options, RenderOptions::new(800, 600));
}

#[test]
fn test_render_options_debug() {
    let options = RenderOptions::new(800, 600);
    let debug_str = format!("{:?}", options);

    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("RenderOptions"));
}

#[test]
fn test_render_options_with_interpolation() {
    let options = RenderOptions::new(800, 600).with_interpolation(Interpolation::Bilinear);
    assert_eq!(options.interpolation(), Interpolation::Bilinear);

    let options = RenderOptions::new(800, 600).with_interpolation(Interpolation::Nearest);
    assert_eq!(options.interpolation(), Interpolation::Nearest);
}

#[test]
fn test_render_options_with_value_range() {
    let options = RenderOptions::new(800, 600).with_value_range(-10.0, 60.0);
    assert_eq!(options.value_range(), Some((-10.0, 60.0)));
}

#[test]
fn test_render_options_with_extent() {
    let extent = GeoExtent {
        min: GeoPoint {
            latitude: 34.0,
            longitude: -99.0,
        },
        max: GeoPoint {
            latitude: 36.0,
            longitude: -97.0,
        },
    };
    let options = RenderOptions::new(800, 600).with_extent(extent);
    assert_eq!(options.extent(), Some(&extent));
}

#[test]
fn test_render_options_with_status_color() {
    let purple = Color::rgb(0.5, 0.0, 0.5);
    let options = RenderOptions::new(800, 600)
        .with_status_color(GateStatus::RangeFolded, Color::WHITE)
        .with_status_color(GateStatus::RangeFolded, purple)
        .with_status_color(GateStatus::NoData, Color::BLACK);

    assert_eq!(options.status_color(GateStatus::RangeFolded), Some(purple));
    assert_eq!(options.status_color(GateStatus::NoData), Some(Color::BLACK));
    assert_eq!(options.status_color(GateStatus::BelowThreshold), None);
}

#[test]
fn test_render_options_status_color_ignores_valid() {
    let options = RenderOptions::new(800, 600).with_status_color(GateStatus::Valid, Color::WHITE);
    assert_eq!(options.status_color(GateStatus::Valid), None);
    assert_eq!(options, RenderOptions::new(800, 600));
}
