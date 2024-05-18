/// A rendered image product.
pub struct Image {
    width: u32,
    height: u32,
    pixels: Vec<Vec<(u8, u8, u8)>>,
}

impl Image {
    /// Create a new image with the given dimensions.
    pub(crate) fn new(width: usize, height: usize) -> Self {
        Self {
            width: width as u32,
            height: height as u32,
            pixels: vec![vec![(0, 0, 0); width]; height],
        }
    }

    /// Get the width of this image.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the height of this image.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Pixel data for this image.
    pub fn pixels(&self) -> &Vec<Vec<(u8, u8, u8)>> {
        &self.pixels
    }

    /// Mutably-access pixel data for this image.
    pub(crate) fn pixels_mut(&mut self) -> &mut Vec<Vec<(u8, u8, u8)>> {
        &mut self.pixels
    }
}
