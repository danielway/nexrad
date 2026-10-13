//! Rendering options.

use crate::{Error, Result};
use nexrad_model::data::SweepField;
use nexrad_model::geo::{GeoExtent, RadarCoordinateSystem};

/// Interpolation method for rendering radar data.
///
/// Controls how pixel values are sampled from the underlying data grid.
/// The default is [`Nearest`](Interpolation::Nearest) for backward compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Interpolation {
    /// Nearest-neighbor sampling (fastest, produces blocky/aliased output).
    ///
    /// Each output pixel takes the value of the single closest data gate.
    /// This is the legacy behavior and the default.
    #[default]
    Nearest,
    /// Bilinear interpolation (smoother, anti-aliased output).
    ///
    /// Each output pixel blends the four surrounding data gates using
    /// bilinear weights. Interpolation happens in **data value space**
    /// (the f32 moment value), not in color space, ensuring scientifically
    /// correct results.
    ///
    /// Falls back to nearest-neighbor for any pixel where one or more of
    /// the four neighbors has a non-[`Valid`](nexrad_model::data::GateStatus::Valid) status.
    Bilinear,
}

/// Options for rendering radar data.
///
/// Use the builder methods to configure rendering options, then pass to
/// [`render_sweep`](crate::render_sweep), [`render_cartesian`](crate::render_cartesian)
/// or [`render_vertical`](crate::render_vertical).
///
/// # Example
///
/// ```
/// use nexrad_render::RenderOptions;
///
/// // Render 800x800 with black background (default)
/// let options = RenderOptions::new(800, 800);
///
/// // Render with transparent background for compositing
/// let options = RenderOptions::new(800, 800).transparent();
///
/// // Render with custom background color (RGBA)
/// let options = RenderOptions::new(800, 800).with_background([255, 255, 255, 255]);
/// ```
#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub(crate) size: (u32, u32),
    pub(crate) background: Option<[u8; 4]>,
    pub(crate) extent: Option<GeoExtent>,
    pub(crate) coord_system: Option<RadarCoordinateSystem>,
    pub(crate) interpolation: Interpolation,
}

impl RenderOptions {
    /// Creates new render options with the specified dimensions and black background.
    ///
    /// Zero dimensions are accepted here; rendering with them returns
    /// [`Error::InvalidDimensions`].
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            size: (width, height),
            background: Some([0, 0, 0, 255]),
            extent: None,
            coord_system: None,
            interpolation: Interpolation::Nearest,
        }
    }

    /// Sets the background to transparent for compositing.
    ///
    /// When rendering with a transparent background, areas without radar data
    /// will be fully transparent, allowing multiple renders to be layered.
    pub fn transparent(mut self) -> Self {
        self.background = None;
        self
    }

    /// Sets a custom background color as RGBA bytes.
    pub fn with_background(mut self, rgba: [u8; 4]) -> Self {
        self.background = Some(rgba);
        self
    }

    /// Sets a geographic extent for the rendered area.
    ///
    /// When set, the image covers exactly this extent. This enables
    /// consistent spatial mapping for side-by-side comparison of multiple renders.
    pub fn with_extent(mut self, extent: GeoExtent) -> Self {
        self.extent = Some(extent);
        self
    }

    /// Sets a radar coordinate system for geographic projection.
    ///
    /// This enables geographic metadata in the [`RenderResult`](crate::RenderResult), including
    /// pixel-to-geo and geo-to-pixel coordinate conversions.
    pub fn with_coord_system(mut self, coord_system: RadarCoordinateSystem) -> Self {
        self.coord_system = Some(coord_system);
        self
    }

    /// Creates render options sized for native resolution of a sweep field.
    ///
    /// Sets both width and height to `gate_count * 2`, which ensures approximately
    /// one pixel per gate at the outer edge of the sweep. This produces the highest
    /// fidelity rendering without wasting pixels.
    /// The size saturates at `u32::MAX` for implausibly large gate counts.
    pub fn native_for(field: &SweepField) -> Self {
        let size = u32::try_from(field.gate_count().saturating_mul(2)).unwrap_or(u32::MAX);
        Self::new(size, size)
    }

    /// Sets the interpolation method for rendering.
    ///
    /// Default is [`Interpolation::Nearest`]. Use [`Interpolation::Bilinear`]
    /// for smoother output that blends neighboring data gates.
    pub fn with_interpolation(mut self, interpolation: Interpolation) -> Self {
        self.interpolation = interpolation;
        self
    }

    /// Shorthand to enable bilinear interpolation.
    ///
    /// Equivalent to `.with_interpolation(Interpolation::Bilinear)`.
    pub fn bilinear(mut self) -> Self {
        self.interpolation = Interpolation::Bilinear;
        self
    }

    /// Output image dimensions (width, height) in pixels.
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    /// Background color as RGBA bytes. `None` means transparent (all zeros).
    pub fn background(&self) -> Option<[u8; 4]> {
        self.background
    }

    /// Geographic extent to render. If `None`, auto-computed from data range.
    ///
    /// When set, the image covers exactly this extent, enabling consistent
    /// spatial mapping across multiple renders for side-by-side comparison.
    pub fn extent(&self) -> Option<&GeoExtent> {
        self.extent.as_ref()
    }

    /// Radar coordinate system for geographic projection.
    ///
    /// When provided, the [`RenderResult`](crate::RenderResult) will include geographic metadata
    /// enabling pixel-to-geo and geo-to-pixel coordinate conversions.
    pub fn coord_system(&self) -> Option<&RadarCoordinateSystem> {
        self.coord_system.as_ref()
    }

    /// Interpolation method for pixel sampling.
    ///
    /// Default is [`Interpolation::Nearest`]. Use [`Interpolation::Bilinear`]
    /// for smoother output that blends neighboring data gates.
    pub fn interpolation(&self) -> Interpolation {
        self.interpolation
    }

    /// Allocates an RGBA pixel buffer for the output image, filled with the background.
    ///
    /// Returns [`Error::InvalidDimensions`] if either dimension is zero or the buffer
    /// size does not fit in memory addressing.
    pub(crate) fn background_buffer(&self) -> Result<Vec<u8>> {
        let (width, height) = self.size;
        if width == 0 || height == 0 {
            return Err(Error::InvalidDimensions);
        }
        let len = (width as usize)
            .checked_mul(height as usize)
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or(Error::InvalidDimensions)?;

        let mut buffer = vec![0u8; len];
        if let Some(bg) = self.background {
            for pixel in buffer.as_chunks_mut::<4>().0 {
                pixel.copy_from_slice(&bg);
            }
        }
        Ok(buffer)
    }
}
