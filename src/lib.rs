mod image;
mod product;

use crate::image::Image;
use crate::product::Product;
use nexrad::model::messages::digital_radar_data;

/// Render a radial's product as an image.
pub fn render_radial(
    radial: &digital_radar_data::Message,
    product: Product,
    size: (usize, usize),
) -> Image {
    todo!()
}
