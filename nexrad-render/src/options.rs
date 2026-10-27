//! Rendering options.

use crate::{Color, ColorLookupTable, ColorScale, Error, Result};
use nexrad_model::data::{GateStatus, Product, SweepField};
use nexrad_model::geo::{GeoExtent, RadarCoordinateSystem};

/// Interpolation method for rendering radar data.
///
/// Controls how pixel values are sampled from the underlying data grid.
/// The default is [`Nearest`](Interpolation::Nearest) for backward compatibility. The enum is
/// non-exhaustive so that another sampling method can be added in a minor release.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
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
/// Use the `with_*` builder methods to configure rendering options, then pass to
/// [`render_sweep`](crate::render_sweep), [`render_cartesian`](crate::render_cartesian)
/// or [`render_vertical`](crate::render_vertical).
///
/// # Example
///
/// ```
/// use nexrad_model::data::GateStatus;
/// use nexrad_render::{Color, Interpolation, RenderOptions};
///
/// // Render 800x800 with black background (default)
/// let options = RenderOptions::new(800, 800);
///
/// // Render with transparent background for compositing
/// let options = RenderOptions::new(800, 800).with_background(Color::TRANSPARENT);
///
/// // Smooth output over a fixed value range, with range-folded gates in purple
/// let options = RenderOptions::new(800, 800)
///     .with_background(Color::WHITE)
///     .with_interpolation(Interpolation::Bilinear)
///     .with_value_range(-32.0, 95.0)
///     .with_status_color(GateStatus::RangeFolded, Color::rgb(0.5, 0.0, 0.5));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct RenderOptions {
    pub(crate) size: (u32, u32),
    background: Color,
    pub(crate) extent: Option<GeoExtent>,
    pub(crate) coord_system: Option<RadarCoordinateSystem>,
    pub(crate) interpolation: Interpolation,
    value_range: Option<(f32, f32)>,
    status_colors: Vec<(GateStatus, Color)>,
}

impl RenderOptions {
    /// Creates new render options with the specified dimensions and black background.
    ///
    /// Zero dimensions are accepted here; rendering with them returns
    /// [`Error::InvalidDimensions`].
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            size: (width, height),
            background: Color::BLACK,
            extent: None,
            coord_system: None,
            interpolation: Interpolation::Nearest,
            value_range: None,
            status_colors: Vec::new(),
        }
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

    /// Sets the background color: the color of pixels without data.
    ///
    /// Use [`Color::TRANSPARENT`] for a transparent background, so that multiple
    /// renders can be layered. A background with zero alpha leaves those pixels
    /// all-zero.
    pub fn with_background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// Sets the geographic extent to render a sweep into.
    ///
    /// [`render_sweep`](crate::render_sweep) then draws the sweep onto a
    /// latitude/longitude grid covering exactly this extent (a
    /// [`Projection::Geographic`](crate::Projection::Geographic) image), so that
    /// renders of different sweeps or radars line up. It requires a
    /// [coordinate system](Self::with_coord_system) to place the radar.
    ///
    /// [`render_cartesian`](crate::render_cartesian) and
    /// [`render_vertical`](crate::render_vertical) ignore this option: their fields
    /// carry their own extents.
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

    /// Sets the interpolation method for rendering.
    ///
    /// Default is [`Interpolation::Nearest`]. Use [`Interpolation::Bilinear`]
    /// for smoother output that blends neighboring data gates.
    pub fn with_interpolation(mut self, interpolation: Interpolation) -> Self {
        self.interpolation = interpolation;
        self
    }

    /// Sets the value range the color lookup table is built over.
    ///
    /// The color scale is sampled from `min` to `max`; values outside the range take
    /// the color at the nearer end. Rendering fails with [`Error::InvalidRange`]
    /// unless both are finite and `min < max`.
    ///
    /// When unset, the range is the field's product range ([`Product::value_range`])
    /// if the field has a product, otherwise the range of the field's valid values,
    /// otherwise `-32.0..95.0`.
    pub fn with_value_range(mut self, min: f32, max: f32) -> Self {
        self.value_range = Some((min, max));
        self
    }

    /// Sets the color for gates with the given non-[`Valid`](GateStatus::Valid)
    /// status, such as purple for [`GateStatus::RangeFolded`].
    ///
    /// By default such gates are drawn as the background. This setting is ignored for
    /// [`GateStatus::Valid`]: valid gates always take their color from the color
    /// scale.
    pub fn with_status_color(mut self, status: GateStatus, color: Color) -> Self {
        if status == GateStatus::Valid {
            return self;
        }
        match self.status_colors.iter_mut().find(|(s, _)| *s == status) {
            Some((_, existing)) => *existing = color,
            None => self.status_colors.push((status, color)),
        }
        self
    }

    /// Output image dimensions (width, height) in pixels.
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    /// Background color, or `None` when it is fully transparent (zero alpha), in
    /// which case pixels without data are all-zero.
    pub fn background(&self) -> Option<Color> {
        (self.background.a > 0.0).then_some(self.background)
    }

    /// Geographic extent to render a sweep into; see [`with_extent`](Self::with_extent).
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

    /// The value range set with [`with_value_range`](Self::with_value_range), or
    /// `None` to use the field-dependent default.
    pub fn value_range(&self) -> Option<(f32, f32)> {
        self.value_range
    }

    /// The color set for gates with `status`, or `None` if they are drawn as the
    /// background. Always `None` for [`GateStatus::Valid`].
    pub fn status_color(&self, status: GateStatus) -> Option<Color> {
        self.status_colors
            .iter()
            .find(|(s, _)| *s == status)
            .map(|(_, color)| *color)
    }

    /// Builds the palette for a field: the color lookup table over the resolved value
    /// range, and the status colors as RGBA bytes.
    ///
    /// The range is the explicit one if set, else the product's range, else the
    /// field's valid-value range unless it is a single value, else `-32.0..95.0`.
    pub(crate) fn palette(
        &self,
        color_scale: &ColorScale,
        product: Option<Product>,
        field_range: Option<(f32, f32)>,
    ) -> Result<Palette> {
        let (min, max) = self
            .value_range
            .or_else(|| product.map(|product| product.value_range()))
            .or_else(|| field_range.filter(|(min, max)| min < max))
            .unwrap_or(DEFAULT_VALUE_RANGE);
        let lut = ColorLookupTable::new(color_scale, min, max, LUT_SIZE)?;
        let status = |status| self.status_color(status).map(|color| color.to_rgba8());
        Ok(Palette {
            lut,
            below_threshold: status(GateStatus::BelowThreshold),
            range_folded: status(GateStatus::RangeFolded),
            no_data: status(GateStatus::NoData),
            filtered: status(GateStatus::Filtered),
        })
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
        if let Some(background) = self.background() {
            let background = background.to_rgba8();
            for pixel in buffer.as_chunks_mut::<4>().0 {
                pixel.copy_from_slice(&background);
            }
        }
        Ok(buffer)
    }
}

/// Value range used when neither the options, the product nor the field give one.
const DEFAULT_VALUE_RANGE: (f32, f32) = (-32.0, 95.0);

/// Number of entries in the color lookup table built for a render.
const LUT_SIZE: usize = 256;

/// The colors a render paints gates with, resolved from [`RenderOptions`].
pub(crate) struct Palette {
    lut: ColorLookupTable,
    below_threshold: Option<[u8; 4]>,
    range_folded: Option<[u8; 4]>,
    no_data: Option<[u8; 4]>,
    filtered: Option<[u8; 4]>,
}

impl Palette {
    /// The color of a valid value.
    #[inline]
    pub(crate) fn value_color(&self, value: f32) -> [u8; 4] {
        self.lut.color(value)
    }

    /// The color of a gate, or `None` if it is drawn as the background.
    #[inline]
    pub(crate) fn gate_color(&self, value: f32, status: GateStatus) -> Option<[u8; 4]> {
        match status {
            GateStatus::Valid => Some(self.lut.color(value)),
            GateStatus::BelowThreshold => self.below_threshold,
            GateStatus::RangeFolded => self.range_folded,
            GateStatus::NoData => self.no_data,
            GateStatus::Filtered => self.filtered,
        }
    }
}
