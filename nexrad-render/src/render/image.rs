//! Rendered image wrapper type.

use piet_common::BitmapTarget;
use std::path::Path;

/// A rendered radar image.
///
/// This wraps the underlying bitmap target and provides convenience methods
/// for working with the rendered output.
///
/// # Example
///
/// ```ignore
/// let mut image = render_polar(&mut device, &sweep, &scale, &opts)?;
///
/// // Save to file
/// image.save_to_file("output.png")?;
///
/// // Get dimensions
/// let (width, height) = image.dimensions();
/// ```
pub struct RenderedImage<'a> {
    target: BitmapTarget<'a>,
    width: usize,
    height: usize,
}

impl<'a> RenderedImage<'a> {
    /// Creates a new RenderedImage from a bitmap target.
    pub(crate) fn new(target: BitmapTarget<'a>, width: usize, height: usize) -> Self {
        Self {
            target,
            width,
            height,
        }
    }

    /// Consumes the image and returns the underlying bitmap target.
    ///
    /// Use this if you need direct access to the piet bitmap target
    /// for advanced operations.
    pub fn into_target(self) -> BitmapTarget<'a> {
        self.target
    }

    /// Returns a reference to the underlying bitmap target.
    pub fn target(&self) -> &BitmapTarget<'a> {
        &self.target
    }

    /// Returns a mutable reference to the underlying bitmap target.
    pub fn target_mut(&mut self) -> &mut BitmapTarget<'a> {
        &mut self.target
    }

    /// Returns the image dimensions (width, height) in pixels.
    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// Returns the image width in pixels.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns the image height in pixels.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Saves the image to a PNG file.
    ///
    /// This consumes the image because the underlying piet API requires
    /// ownership to save.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn save_to_file(self, path: impl AsRef<Path>) -> Result<(), piet::Error> {
        self.target.save_to_file(path)
    }

    /// Copies the image data to a byte buffer.
    ///
    /// The buffer must be large enough to hold the image data in the
    /// specified format. Returns the number of bytes written.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is too small or the copy fails.
    pub fn copy_raw_pixels(
        &mut self,
        format: piet_common::ImageFormat,
        buf: &mut [u8],
    ) -> Result<usize, piet::Error> {
        self.target.copy_raw_pixels(format, buf)
    }
}
