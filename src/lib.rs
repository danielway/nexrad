use nexrad::model::messages::digital_radar_data;
use nexrad::model::messages::digital_radar_data::{GenericDataBlock, ScaledMomentValue};
use piet::{Color, RenderContext};
use piet_common::kurbo::{Arc, Point, Vec2};
use piet_common::{BitmapTarget, Device};
use std::cmp::max;
use uom::si::angle::radian;

/// Radar data products to render.
#[derive(Debug, Copy, Clone)]
pub enum Product {
    Reflectivity,
    Velocity,
    SpectrumWidth,
    DifferentialReflectivity,
    DifferentialPhase,
    CorrelationCoefficient,
    SpecificDiffPhase,
}

/// Render the specified radials to an image.
pub fn render_radial<'a>(
    device: &'a mut Device,
    radials: &Vec<digital_radar_data::Message>,
    product: Product,
    size: (usize, usize),
) -> BitmapTarget<'a> {
    let mut target = device
        .bitmap_target(size.0, size.1, 1.0)
        .expect("create bitmap target");

    let mut render_context = target.render_context();
    render_context.clear(None, Color::BLACK);

    let image_center = Point::new(size.0 as f64 / 2.0, size.1 as f64 / 2.0);

    for radial in radials {
        let azimuth = radial.header.azimuth_angle();
        let data_moment = get_radial_moment(product, radial);

        let first_gate_distance = data_moment.header.data_moment_range();

        for (gate_index, scaled_gate) in data_moment.decoded_values().into_iter().enumerate() {
            if let ScaledMomentValue::Value(value) = scaled_gate {
                let gate_interval = data_moment.header.data_moment_range();

                let radar_range = first_gate_distance
                    + data_moment.header.number_of_data_moment_gates as f64 * gate_interval;

                let render_scale = max(size.0, size.1) as f64 / 2.0 / radar_range;

                let gate_distance = first_gate_distance + gate_index as f64 * gate_interval;

                let scaled_gate_interval = gate_interval * render_scale;

                // todo: why do we subtract half an interval instead of adding?
                let gate_midpoint = gate_distance - gate_interval / 2.0;
                let scaled_gate_midpoint = render_scale * gate_midpoint;

                render_context.stroke(
                    Arc::new(
                        image_center,
                        Vec2::new(scaled_gate_midpoint.value, scaled_gate_midpoint.value),
                        azimuth.get::<radian>(),
                        radial.header.azimuth_resolution_spacing().get::<radian>(),
                        0.0,
                    ),
                    &get_color(value, ""),
                    scaled_gate_interval.value,
                );
            }
        }
    }

    render_context.finish().expect("completed render");
    drop(render_context);

    target
}

/// Retrieve the generic data block in this radial for the given product, panicking if unavailable.
fn get_radial_moment(product: Product, radial: &digital_radar_data::Message) -> &GenericDataBlock {
    match product {
        Product::Reflectivity => &radial.reflectivity_data_block,
        Product::Velocity => &radial.velocity_data_block,
        Product::SpectrumWidth => &radial.spectrum_width_data_block,
        Product::DifferentialReflectivity => &radial.differential_reflectivity_data_block,
        Product::DifferentialPhase => &radial.differential_phase_data_block,
        Product::CorrelationCoefficient => &radial.correlation_coefficient_data_block,
        Product::SpecificDiffPhase => &radial.specific_diff_phase_data_block,
    }
    .as_ref()
    .expect("has requested product moment")
}

fn get_color(value: f32, _product: &str) -> Color {
    // todo: separate color schemes for each product
    // todo: make colors vary continuously, rather than in discrete steps
    if value < 5.0 {
        Color::rgb8(0, 0, 0)
    } else if value >= 5.0 && value < 10.0 {
        Color::rgb8(0x40, 0xe8, 0xe3)
    } else if value >= 10.0 && value < 15.0 {
        Color::rgb8(0x26, 0xa4, 0xfa)
    } else if value >= 15.0 && value < 20.0 {
        Color::rgb8(0x00, 0x30, 0xed)
    } else if value >= 20.0 && value < 25.0 {
        Color::rgb8(0x49, 0xfb, 0x3e)
    } else if value >= 25.0 && value < 30.0 {
        Color::rgb8(0x36, 0xc2, 0x2e)
    } else if value >= 30.0 && value < 35.0 {
        Color::rgb8(0x27, 0x8c, 0x1e)
    } else if value >= 35.0 && value < 40.0 {
        Color::rgb8(0xfe, 0xf5, 0x43)
    } else if value >= 40.0 && value < 45.0 {
        Color::rgb8(0xeb, 0xb4, 0x33)
    } else if value >= 45.0 && value < 50.0 {
        Color::rgb8(0xf6, 0x95, 0x2e)
    } else if value >= 50.0 && value < 55.0 {
        Color::rgb8(0xf8, 0x0a, 0x26)
    } else if value >= 55.0 && value < 60.0 {
        Color::rgb8(0xcb, 0x05, 0x16)
    } else if value >= 60.0 && value < 65.0 {
        Color::rgb8(0xa9, 0x08, 0x13)
    } else if value >= 65.0 && value < 70.0 {
        Color::rgb8(0xee, 0x34, 0xfa)
    } else {
        Color::rgb8(0xff, 0xff, 0xFF)
    }
}
