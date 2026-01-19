use nexrad_decode::messages::MessageContents;
use nexrad_render::{get_nws_reflectivity_scale, render_radials, Product};
use piet_common::Device;
use std::fs::File;
use std::io::Read;

fn main() {
    let file_name = "KDMX20220305_232324_V06";

    let mut file = File::open(format!("downloads/{file_name}")).expect("file exists");
    let mut data = Vec::new();
    file.read_to_end(&mut data).expect("can read file");

    let archive = nexrad_data::volume::File::new(data);

    let target_elevation_number = 1; // TODO: make this configurable
    let mut radials = Vec::new();
    for mut record in archive.records().expect("records") {
        if record.compressed() {
            record = record.decompress().expect("decompresses successfully");
        }

        for message in record.messages().expect("messages are valid") {
            if let MessageContents::DigitalRadarData(message) = message.contents() {
                if message.header().elevation_number() == target_elevation_number {
                    radials.push(message.radial().expect("radial is valid"));
                }
            }
        }
    }

    let mut device = Device::new().expect("created device");
    let color_scale = get_nws_reflectivity_scale();

    let mut render_product = |product: Product| {
        let image = render_radials(&mut device, &radials, product, &color_scale, (3000, 3000))
            .expect("renders successfully");

        std::fs::create_dir_all("renders").expect("creates directory");

        image
            .save_to_file(format!("renders/{file_name}_{product:?}.png"))
            .expect("saves to file");
    };

    render_product(Product::Reflectivity);
    // render_product(Product::Velocity);
    // render_product(Product::SpectrumWidth);
}
