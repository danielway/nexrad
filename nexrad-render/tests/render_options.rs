//! Tests for RenderOptions.

use nexrad_render::RenderOptions;
use piet::Color;

#[test]
fn test_render_options_new() {
    let options = RenderOptions::new(800, 600);

    assert_eq!(options.size, (800, 600));
    // Default background should be black
    assert_eq!(options.background, Some(Color::BLACK));
}

#[test]
fn test_render_options_transparent() {
    let options = RenderOptions::new(800, 600).transparent();

    assert_eq!(options.size, (800, 600));
    assert_eq!(options.background, None);
}

#[test]
fn test_render_options_with_background() {
    let options = RenderOptions::new(800, 600).with_background(Color::WHITE);

    assert_eq!(options.size, (800, 600));
    assert_eq!(options.background, Some(Color::WHITE));
}

#[test]
fn test_render_options_chain() {
    // Test that builder methods can be chained
    let options = RenderOptions::new(1000, 1000)
        .transparent()
        .with_background(Color::rgb(0.5, 0.5, 0.5));

    assert_eq!(options.size, (1000, 1000));
    // Last call wins
    assert_eq!(options.background, Some(Color::rgb(0.5, 0.5, 0.5)));
}

#[test]
fn test_render_options_clone() {
    let options = RenderOptions::new(800, 600).transparent();
    let cloned = options.clone();

    assert_eq!(options.size, cloned.size);
    assert_eq!(options.background, cloned.background);
}

#[test]
fn test_render_options_debug() {
    let options = RenderOptions::new(800, 600);
    let debug_str = format!("{:?}", options);

    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("RenderOptions"));
}
