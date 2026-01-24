//! Tests for RenderOptions.

use nexrad_render::RenderOptions;

#[test]
fn test_render_options_new() {
    let options = RenderOptions::new(800, 600);

    assert_eq!(options.size, (800, 600));
    // Default background should be black (opaque)
    assert_eq!(options.background, Some([0, 0, 0, 255]));
}

#[test]
fn test_render_options_transparent() {
    let options = RenderOptions::new(800, 600).transparent();

    assert_eq!(options.size, (800, 600));
    assert_eq!(options.background, None);
}

#[test]
fn test_render_options_with_background() {
    let options = RenderOptions::new(800, 600).with_background([255, 255, 255, 255]);

    assert_eq!(options.size, (800, 600));
    assert_eq!(options.background, Some([255, 255, 255, 255]));
}

#[test]
fn test_render_options_chain() {
    // Test that builder methods can be chained
    let options = RenderOptions::new(1000, 1000)
        .transparent()
        .with_background([128, 128, 128, 255]);

    assert_eq!(options.size, (1000, 1000));
    // Last call wins
    assert_eq!(options.background, Some([128, 128, 128, 255]));
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
