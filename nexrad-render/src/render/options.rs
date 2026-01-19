//! Render options for radar visualization.

use piet::Color;

/// Options for rendering radar data to an image.
///
/// Use the builder methods to configure rendering options.
///
/// # Example
///
/// ```
/// use nexrad_render::render::RenderOpts;
/// use piet::Color;
///
/// // Default: 800x800 with black background
/// let opts = RenderOpts::new(800, 800);
///
/// // Transparent background for compositing
/// let opts = RenderOpts::new(800, 800).transparent();
///
/// // Custom max range for zooming
/// let opts = RenderOpts::new(800, 800).with_max_range(150_000.0);
///
/// // Show NoData areas with a color
/// let opts = RenderOpts::new(800, 800).with_nodata_color(Color::grey8(32));
/// ```
#[derive(Debug, Clone)]
pub struct RenderOpts {
    /// Output image width in pixels.
    pub width: usize,
    /// Output image height in pixels.
    pub height: usize,
    /// Background color. `None` means transparent.
    pub background: Option<Color>,
    /// Maximum range to render in meters. `None` means auto from data.
    pub max_range_m: Option<f32>,
    /// Color for NoData/invalid samples. `None` means transparent.
    pub nodata_color: Option<Color>,
}

impl RenderOpts {
    /// Creates new render options with the given dimensions and black background.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            background: Some(Color::BLACK),
            max_range_m: None,
            nodata_color: None,
        }
    }

    /// Sets the background to transparent for compositing.
    pub fn transparent(mut self) -> Self {
        self.background = None;
        self
    }

    /// Sets a custom background color.
    pub fn with_background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    /// Sets the maximum range in meters.
    ///
    /// This controls how much of the radar data is visible. Data beyond
    /// this range will be clipped. If not set, the range is determined
    /// automatically from the data.
    pub fn with_max_range(mut self, range_m: f32) -> Self {
        self.max_range_m = Some(range_m);
        self
    }

    /// Sets the color for NoData pixels.
    ///
    /// When set, pixels that have no data (NaN values) will be rendered
    /// with this color instead of being transparent.
    pub fn with_nodata_color(mut self, color: Color) -> Self {
        self.nodata_color = Some(color);
        self
    }

    /// Returns the output dimensions (width, height).
    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }
}

impl Default for RenderOpts {
    fn default() -> Self {
        Self::new(800, 800)
    }
}
