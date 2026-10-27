//! Tests for render errors and the projection reported by each render entry point.

use nexrad_model::data::{
    CartesianField, GateStatus, MomentData, Product, Radial, SweepField, VerticalField,
};
use nexrad_model::geo::{GeoExtent, GeoPoint};
use nexrad_render::{
    default_scale, render_cartesian, render_sweep, render_vertical, Error, Projection,
    RenderOptions,
};

fn sweep_field() -> SweepField {
    let radials: Vec<Radial> = (0..360)
        .map(|az| {
            let moment = MomentData::from_fixed_point(
                Product::Reflectivity,
                100,
                2000,
                250,
                8,
                2.0,
                66.0,
                vec![100; 100],
            )
            .unwrap();
            Radial::builder(0, az as f32, 1, 0.5)
                .moment(moment)
                .build()
                .unwrap()
        })
        .collect();
    SweepField::from_radials(&radials, Product::Reflectivity).expect("field is created")
}

fn extent() -> GeoExtent {
    GeoExtent {
        min: GeoPoint {
            latitude: 40.0,
            longitude: -94.0,
        },
        max: GeoPoint {
            latitude: 42.0,
            longitude: -92.0,
        },
    }
}

fn cartesian_field(width: usize, height: usize) -> CartesianField {
    let mut field = CartesianField::new("Test", "dBZ", extent(), width, height).unwrap();
    for row in 0..height {
        for col in 0..width {
            field.set(row, col, 30.0, GateStatus::Valid).unwrap();
        }
    }
    field
}

fn vertical_field(width: usize, height: usize) -> VerticalField {
    let mut field =
        VerticalField::new("Test", "dBZ", (0.0, 100.0), (0.0, 10000.0), width, height).unwrap();
    for row in 0..height {
        for col in 0..width {
            field.set(row, col, 30.0, GateStatus::Valid).unwrap();
        }
    }
    field
}

#[test]
fn zero_dimensions_are_rejected_by_every_renderer() {
    let scale = default_scale(Product::Reflectivity);

    for options in [RenderOptions::new(0, 100), RenderOptions::new(100, 0)] {
        assert!(matches!(
            render_sweep(&sweep_field(), &scale, &options),
            Err(Error::InvalidDimensions)
        ));
        assert!(matches!(
            render_cartesian(&cartesian_field(4, 4), &scale, &options),
            Err(Error::InvalidDimensions)
        ));
        assert!(matches!(
            render_vertical(&vertical_field(4, 4), &scale, &options),
            Err(Error::InvalidDimensions)
        ));
    }
}

#[test]
fn empty_grid_fields_cannot_be_constructed() {
    // The model rejects zero-sized grids, so the renderers never receive one from a constructor.
    assert!(matches!(
        CartesianField::new("Test", "dBZ", extent(), 0, 4),
        Err(nexrad_model::result::Error::InvalidGeometry {
            parameter: "width",
            ..
        })
    ));
    assert!(matches!(
        VerticalField::new("Test", "dBZ", (0.0, 100.0), (0.0, 10000.0), 4, 0),
        Err(nexrad_model::result::Error::InvalidGeometry {
            parameter: "height",
            ..
        })
    ));
}

#[test]
fn render_sweep_reports_polar_projection() {
    let field = sweep_field();
    let scale = default_scale(Product::Reflectivity);
    let result = render_sweep(&field, &scale, &RenderOptions::new(200, 100)).expect("renders");
    let meta = result.metadata();

    assert_eq!((meta.width(), meta.height()), (200, 100));
    let Projection::Polar {
        center_pixel,
        pixels_per_km,
        max_range_km,
        ..
    } = *meta.projection()
    else {
        panic!("expected a polar projection, got {:?}", meta.projection());
    };
    assert_eq!(center_pixel, (100.0, 50.0));
    assert_eq!(max_range_km, field.max_range_km());
    assert_eq!(pixels_per_km, 100.0 / field.max_range_km());
    assert_eq!(meta.center_pixel(), Some(center_pixel));
    assert!(meta.geo_extent().is_none());

    let query = result
        .metadata()
        .query_polar(&field, 90.0, 10.0)
        .expect("point is inside the field");
    assert_eq!(query.status, GateStatus::Valid);
}

#[test]
fn render_cartesian_reports_geographic_projection() {
    let field = cartesian_field(4, 4);
    let scale = default_scale(Product::Reflectivity);
    let result = render_cartesian(&field, &scale, &RenderOptions::new(16, 16)).expect("renders");
    let meta = result.metadata();

    let Projection::Geographic { extent: actual, .. } = meta.projection() else {
        panic!(
            "expected a geographic projection, got {:?}",
            meta.projection()
        );
    };
    assert_eq!(actual, &extent());
    assert_eq!(meta.geo_extent(), Some(&extent()));
    assert!(meta.pixels_per_km().is_none());
    assert!(meta.pixel_to_polar(8.0, 8.0).is_none());

    let center = meta.pixel_to_geo(8.0, 8.0).expect("center is inside");
    assert!((center.latitude - 41.0).abs() < 1e-9);
    assert!((center.longitude + 93.0).abs() < 1e-9);
}

#[test]
fn render_vertical_reports_vertical_projection() {
    let field = vertical_field(4, 4);
    let scale = default_scale(Product::Reflectivity);
    let result = render_vertical(&field, &scale, &RenderOptions::new(16, 16)).expect("renders");
    let meta = result.metadata();

    let Projection::Vertical {
        distance_range_km,
        altitude_range_meters,
        ..
    } = *meta.projection()
    else {
        panic!(
            "expected a vertical projection, got {:?}",
            meta.projection()
        );
    };
    assert_eq!(distance_range_km, (0.0, 100.0));
    assert_eq!(altitude_range_meters, (0.0, 10000.0));
    assert!(meta.geo_extent().is_none());
    assert!(meta.pixel_to_polar(8.0, 8.0).is_none());
    assert!(meta.pixel_to_geo(8.0, 8.0).is_none());
}

#[test]
fn filtered_gates_render_as_background() {
    const BACKGROUND: [u8; 4] = [1, 2, 3, 255];
    let scale = default_scale(Product::Reflectivity);
    let options = RenderOptions::new(100, 100).with_background(BACKGROUND.into());
    let all_background = |result: &nexrad_render::RenderResult| {
        result
            .pixels()
            .as_chunks::<4>()
            .0
            .iter()
            .all(|p| *p == BACKGROUND)
    };

    let mut sweep = sweep_field();
    for az in 0..sweep.azimuth_count() {
        for gate in 0..sweep.gate_count() {
            sweep.set(az, gate, 40.0, GateStatus::Filtered).unwrap();
        }
    }
    let result = render_sweep(&sweep, &scale, &options).expect("renders");
    assert!(all_background(&result));

    let query = result
        .metadata()
        .query_pixel(&sweep, 70.0, 50.0)
        .expect("pixel is inside the field");
    assert_eq!(query.status, GateStatus::Filtered);
    assert_eq!(query.value, 40.0);

    let mut cartesian = cartesian_field(4, 4);
    let mut vertical = vertical_field(4, 4);
    for row in 0..4 {
        for col in 0..4 {
            cartesian.set(row, col, 40.0, GateStatus::Filtered).unwrap();
            vertical.set(row, col, 40.0, GateStatus::Filtered).unwrap();
        }
    }
    let result = render_cartesian(&cartesian, &scale, &options).expect("renders");
    assert!(all_background(&result));
    let result = render_vertical(&vertical, &scale, &options).expect("renders");
    assert!(all_background(&result));
}
