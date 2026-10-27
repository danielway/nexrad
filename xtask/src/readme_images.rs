//! Renders the images shown in the READMEs.
//!
//! Every image is drawn from one public NOAA volume through the crates' public API, so the
//! READMEs show what the library produces. The volume is cached under `downloads/`, and the
//! images are written to `docs/images/`.

use crate::Result;
use nexrad::model::data::{Product, Scan, SweepField};
use nexrad::model::geo::{GeoExtent, GeoPoint, RadarCoordinateSystem};
use nexrad::process::filter::{CorrelationCoefficientFilter, MedianFilter};
use nexrad::process::SweepPipeline;
use nexrad::render::{default_scale, render_sweep, Color, Interpolation, RenderOptions};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;

/// KDMX (Des Moines, Iowa), 5 March 2022 23:23 UTC. The showcase example uses the same scan.
const VOLUME_NAME: &str = "KDMX20220305_232324_V06";
const VOLUME_URL: &str =
    "https://unidata-nexrad-level2.s3.amazonaws.com/2022/03/05/KDMX/KDMX20220305_232324_V06";
const CACHE_DIR: &str = "downloads";
const OUTPUT_DIR: &str = "docs/images";

/// Black, to match the lowest step of the reflectivity scale.
const BACKGROUND: [u8; 4] = [0, 0, 0, 255];
const TILE_SIZE: u32 = 400;
const GAP: u32 = 6;
/// Grey between grid tiles, so the black tiles read as separate images.
const GAP_COLOUR: [u8; 4] = [72, 79, 88, 255];

/// The products in the product grid, in reading order.
const GRID_PRODUCTS: [Product; 6] = [
    Product::Reflectivity,
    Product::Velocity,
    Product::SpectrumWidth,
    Product::DifferentialReflectivity,
    Product::CorrelationCoefficient,
    Product::DifferentialPhase,
];

/// Gates whose correlation coefficient is below this are removed in the processing image.
const CORRELATION_THRESHOLD: f32 = 0.8;

pub async fn generate() -> Result<()> {
    let scan = nexrad::load(load_volume().await?)?;
    let coords = scan
        .coordinate_system()
        .ok_or("the volume has no site metadata")?;
    fs::create_dir_all(OUTPUT_DIR)?;

    let reflectivity = lowest(&scan, Product::Reflectivity)?;
    let hero = render(
        Product::Reflectivity,
        &reflectivity,
        &coords,
        box_around(&coords, (0.1, -0.3), 1.3, 2.8),
        1200,
        750,
    )?;
    write_png("reflectivity.png", &hero)?;

    let tile_extent = box_around(&coords, (0.0, -0.35), 1.85, 2.5);
    let tiles = GRID_PRODUCTS
        .iter()
        .map(|&product| {
            render(
                product,
                &lowest(&scan, product)?,
                &coords,
                tile_extent,
                TILE_SIZE,
                TILE_SIZE,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    write_png("products.png", &grid(&tiles, 3))?;

    let filtered = SweepPipeline::new()
        .then(CorrelationCoefficientFilter::new(
            CORRELATION_THRESHOLD,
            lowest(&scan, Product::CorrelationCoefficient)?,
        )?)
        .then(MedianFilter::new(3, 3)?)
        .execute(&reflectivity)?;
    let pair_extent = box_around(&coords, (0.0, -0.15), 1.2, 1.6);
    let pair = [
        render(
            Product::Reflectivity,
            &reflectivity,
            &coords,
            pair_extent,
            TILE_SIZE,
            TILE_SIZE,
        )?,
        render(
            Product::Reflectivity,
            &filtered,
            &coords,
            pair_extent,
            TILE_SIZE,
            TILE_SIZE,
        )?,
    ];
    write_png("processing.png", &grid(&pair, 2))?;

    Ok(())
}

async fn load_volume() -> Result<Vec<u8>> {
    let path = Path::new(CACHE_DIR).join(VOLUME_NAME);
    if path.exists() {
        return Ok(fs::read(path)?);
    }
    println!("Downloading {VOLUME_URL}");
    let client = reqwest::Client::builder()
        .user_agent("nexrad-xtask readme-images")
        .timeout(Duration::from_secs(120))
        .build()?;
    let bytes = client
        .get(VOLUME_URL)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?
        .to_vec();
    fs::create_dir_all(CACHE_DIR)?;
    fs::write(path, &bytes)?;
    Ok(bytes)
}

/// The product from the first sweep that carries it. Products split across the two scans of
/// the lowest tilt come from whichever scan has them.
fn lowest(scan: &Scan, product: Product) -> Result<SweepField> {
    scan.sweeps()
        .iter()
        .find_map(|sweep| sweep.field(product))
        .ok_or_else(|| format!("the volume has no {} data", product.as_str()).into())
}

/// A latitude/longitude box centred `north` degrees north and `east` degrees east of the radar.
fn box_around(
    coords: &RadarCoordinateSystem,
    (north, east): (f64, f64),
    latitude_span: f64,
    longitude_span: f64,
) -> GeoExtent {
    let latitude = coords.latitude() + north;
    let longitude = coords.longitude() + east;
    GeoExtent {
        min: GeoPoint {
            latitude: latitude - latitude_span / 2.0,
            longitude: longitude - longitude_span / 2.0,
        },
        max: GeoPoint {
            latitude: latitude + latitude_span / 2.0,
            longitude: longitude + longitude_span / 2.0,
        },
    }
}

struct Image {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

fn render(
    product: Product,
    field: &SweepField,
    coords: &RadarCoordinateSystem,
    extent: GeoExtent,
    width: u32,
    height: u32,
) -> Result<Image> {
    let options = RenderOptions::new(width, height)
        .with_background(Color::from_rgba8(BACKGROUND))
        .with_coord_system(coords.clone())
        .with_extent(extent)
        .with_interpolation(Interpolation::Bilinear);
    let result = render_sweep(field, &default_scale(product), &options)?;
    Ok(Image {
        width: result.width(),
        height: result.height(),
        pixels: result.into_pixels(),
    })
}

/// Lays equally sized images out in rows of `columns`, separated by grey gaps.
fn grid(images: &[Image], columns: u32) -> Image {
    let (tile_width, tile_height) = (images[0].width, images[0].height);
    let rows = (images.len() as u32).div_ceil(columns);
    let width = columns * tile_width + (columns - 1) * GAP;
    let height = rows * tile_height + (rows - 1) * GAP;
    let mut pixels = GAP_COLOUR.repeat((width * height) as usize);
    for (index, image) in images.iter().enumerate() {
        let left = (index as u32 % columns) * (tile_width + GAP);
        let top = (index as u32 / columns) * (tile_height + GAP);
        for y in 0..tile_height {
            let source = (y * tile_width * 4) as usize;
            let target = (((top + y) * width + left) * 4) as usize;
            let row = (tile_width * 4) as usize;
            pixels[target..target + row].copy_from_slice(&image.pixels[source..source + row]);
        }
    }
    Image {
        width,
        height,
        pixels,
    }
}

/// Writes an opaque image as an indexed PNG when it has at most 256 colours, which keeps
/// the README images small, and as RGB otherwise.
fn write_png(name: &str, image: &Image) -> Result<()> {
    let path = Path::new(OUTPUT_DIR).join(name);
    let mut encoder = png::Encoder::new(fs::File::create(&path)?, image.width, image.height);
    encoder.set_compression(png::Compression::High);

    let mut palette: HashMap<[u8; 3], u8> = HashMap::new();
    let mut indices = Vec::with_capacity((image.width * image.height) as usize);
    for pixel in image.pixels.as_chunks::<4>().0 {
        let rgb = [pixel[0], pixel[1], pixel[2]];
        let next = palette.len();
        let index = *palette.entry(rgb).or_insert(next.min(255) as u8);
        indices.push(index);
    }

    if palette.len() <= 256 {
        let mut table = vec![0; palette.len() * 3];
        for (rgb, index) in &palette {
            table[*index as usize * 3..*index as usize * 3 + 3].copy_from_slice(rgb);
        }
        encoder.set_color(png::ColorType::Indexed);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_palette(table);
        encoder.write_header()?.write_image_data(&indices)?;
    } else {
        let rgb: Vec<u8> = image
            .pixels
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|pixel| [pixel[0], pixel[1], pixel[2]])
            .collect();
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.write_header()?.write_image_data(&rgb)?;
    }

    println!("Wrote {} ({} colours)", path.display(), palette.len());
    Ok(())
}
