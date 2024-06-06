use nexrad::decompress::decompress_and_decode_archive2_file;
use nexrad::model::messages::Message;
use nexrad_renderer::{render_radial, Product};
use piet_common::Device;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Cursor, Read};

fn main() {
    let file_name = "KDMX20220305_221051_V06";

    let mut file = File::open(format!("files/{}", file_name)).expect("file exists");
    let mut data = Vec::new();
    file.read_to_end(&mut data).expect("can read file");

    let file_size = data.len();
    let mut cursor = Cursor::new(data);

    let archive = decompress_and_decode_archive2_file(&mut cursor, file_size as u64)
        .expect("decompresses and decodes successfully");

    let messages_by_elevation = archive
        .messages
        .into_iter()
        .filter(|m| match m.message {
            Message::DigitalRadarData(_) => true,
            _ => false,
        })
        .fold(HashMap::new(), |mut groups, message| {
            if let Message::DigitalRadarData(message) = message.message {
                if !groups.contains_key(&message.header.elevation_number) {
                    groups.insert(message.header.elevation_number, Vec::new());
                }
                groups
                    .get_mut(&message.header.elevation_number)
                    .expect("elevation exists")
                    .push(message);
            }
            groups
        });

    let mut elevations = messages_by_elevation.keys().collect::<Vec<_>>();
    elevations.sort();
    let arbitrary_elevation = elevations.get(1).unwrap();
    let arbitrary_elevation_messages = messages_by_elevation
        .get(arbitrary_elevation)
        .expect("elevation exists");

    let products =
        arbitrary_elevation_messages
            .iter()
            .fold(HashSet::new(), |mut products, message| {
                if message.volume_data_block.is_some() {
                    products.insert("volume");
                }
                if message.elevation_data_block.is_some() {
                    products.insert("elevation");
                }
                if message.radial_data_block.is_some() {
                    products.insert("radial");
                }
                if message.reflectivity_data_block.is_some() {
                    products.insert("reflectivity");
                }
                if message.velocity_data_block.is_some() {
                    products.insert("velocity");
                }
                if message.spectrum_width_data_block.is_some() {
                    products.insert("spectrum_width");
                }
                if message.differential_reflectivity_data_block.is_some() {
                    products.insert("differential_reflectivity");
                }
                if message.differential_phase_data_block.is_some() {
                    products.insert("differential_phase");
                }
                if message.correlation_coefficient_data_block.is_some() {
                    products.insert("correlation_coefficient");
                }
                if message.specific_diff_phase_data_block.is_some() {
                    products.insert("specific_diff_phase");
                }
                products
            });

    println!("{:?}", products);

    let mut device = Device::new().expect("created device");

    let mut render_product = |product: Product| {
        let image = render_radial(
            &mut device,
            arbitrary_elevation_messages,
            product,
            (1000, 1000),
        );
        image
            .save_to_file(format!("renders/{}_{:?}.png", file_name, product))
            .expect("saves to file");
    };

    render_product(Product::Reflectivity);
    render_product(Product::Velocity);
    render_product(Product::SpectrumWidth);
}
