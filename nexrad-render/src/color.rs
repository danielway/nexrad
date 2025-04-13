use piet::Color;

pub struct ColorScaleLevel {
    value_dbz: f32,
    color: Color,
}

impl ColorScaleLevel {
    pub fn new(value_dbz: f32, color: Color) -> Self {
        Self { value_dbz, color }
    }
}

pub struct DiscreteColorScale {
    levels: Vec<ColorScaleLevel>,
}

impl DiscreteColorScale {
    pub fn new(mut levels: Vec<ColorScaleLevel>) -> Self {
        levels.sort_by(|a, b| b.value_dbz.total_cmp(&a.value_dbz));
        Self { levels }
    }

    pub fn get_color(&self, value_dbz: f32) -> Color {
        let mut color = Color::BLACK;

        for level in &self.levels {
            if value_dbz >= level.value_dbz {
                return level.color;
            }

            color = level.color;
        }

        color
    }
}

pub fn get_nws_reflectivity_scale() -> DiscreteColorScale {
    DiscreteColorScale::new(vec![
        ColorScaleLevel::new(0.0, Color::rgb(0.0000, 0.0000, 0.0000)),
        ColorScaleLevel::new(5.0, Color::rgb(0.0000, 1.0000, 1.0000)),
        ColorScaleLevel::new(10.0, Color::rgb(0.5294, 0.8078, 0.9216)),
        ColorScaleLevel::new(15.0, Color::rgb(0.0000, 0.0000, 1.0000)),
        ColorScaleLevel::new(20.0, Color::rgb(0.0000, 1.0000, 0.0000)),
        ColorScaleLevel::new(25.0, Color::rgb(0.1961, 0.8039, 0.1961)),
        ColorScaleLevel::new(30.0, Color::rgb(0.1333, 0.5451, 0.1333)),
        ColorScaleLevel::new(35.0, Color::rgb(0.9333, 0.9333, 0.0000)),
        ColorScaleLevel::new(40.0, Color::rgb(0.9333, 0.8627, 0.5098)),
        ColorScaleLevel::new(45.0, Color::rgb(0.9333, 0.4627, 0.1294)),
        ColorScaleLevel::new(50.0, Color::rgb(1.0000, 0.1882, 0.1882)),
        ColorScaleLevel::new(55.0, Color::rgb(0.6902, 0.1882, 0.3765)),
        ColorScaleLevel::new(60.0, Color::rgb(0.6902, 0.1882, 0.3765)),
        ColorScaleLevel::new(65.0, Color::rgb(0.7294, 0.3333, 0.8275)),
        ColorScaleLevel::new(70.0, Color::rgb(1.0000, 0.0000, 1.0000)),
        ColorScaleLevel::new(75.0, Color::rgb(1.0000, 1.0000, 1.0000)),
    ])
}
