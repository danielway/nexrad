//! Tests for Product enum.

use nexrad_render::Product;

#[test]
fn test_product_equality() {
    assert_eq!(Product::Reflectivity, Product::Reflectivity);
    assert_ne!(Product::Reflectivity, Product::Velocity);
}

#[test]
fn test_product_copy() {
    let p1 = Product::Reflectivity;
    let p2 = p1; // Copy
    assert_eq!(p1, p2);
}

#[test]
fn test_product_clone() {
    let p1 = Product::Velocity;
    let p2 = p1.clone();
    assert_eq!(p1, p2);
}

#[test]
fn test_product_debug() {
    let debug_str = format!("{:?}", Product::Reflectivity);
    assert_eq!(debug_str, "Reflectivity");

    let debug_str = format!("{:?}", Product::DifferentialReflectivity);
    assert_eq!(debug_str, "DifferentialReflectivity");
}

#[test]
fn test_product_hash() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(Product::Reflectivity);
    set.insert(Product::Velocity);
    set.insert(Product::Reflectivity); // Duplicate

    assert_eq!(set.len(), 2);
    assert!(set.contains(&Product::Reflectivity));
    assert!(set.contains(&Product::Velocity));
}
