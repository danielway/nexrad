use nexrad_decode::messages::MessageContents;
use nexrad_render::{get_nws_reflectivity_scale, render_radials, Product, RenderOptions};
use std::fs::File;
use std::io::Read;

fn main() {
    let file_name = "KDMX20220305_232324_V06";

    let mut file = File::open(format!("downloads/{file_name}")).expect("file exists");
    let mut data = Vec::new();
    file.read_to_end(&mut data).expect("can read file");

    let archive = nexrad_data::volume::File::new(data);

    let target_elevation_number = 1;
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

    let color_scale = get_nws_reflectivity_scale();
    let options = RenderOptions::new(3000, 3000);

    let render_product = |product: Product| {
        let image = render_radials(&radials, product, &color_scale, &options)
            .expect("renders successfully");

        std::fs::create_dir_all("renders").expect("creates directory");

        image
            .save(format!("renders/{file_name}_{product:?}.png"))
            .expect("saves image");

        println!(
            "Rendered {product:?}: {} x {} pixels",
            image.width(),
            image.height()
        );
    };

    render_product(Product::Reflectivity);
}
