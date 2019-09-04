#[allow(dead_code)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Color {
    r: f32,
    b: f32,
    g: f32,
}

impl Color {
    #[allow(dead_code)]
    pub fn new(r: f32, b: f32, g: f32) -> Self {
        Color { r, g, b }
    }

    #[allow(dead_code)]
    pub fn with_u8(r: u8, g: u8, b: u8) -> Self {
        Color {
            r: Color::into_linear(r as f32 / 255.0),
            g: Color::into_linear(g as f32 / 255.0),
            b: Color::into_linear(b as f32 / 255.0),
        }
    }

    #[allow(dead_code)]
    pub fn into_raw_u32(self) -> [u32; 3] {
        [self.r.to_bits(), self.g.to_bits(), self.b.to_bits()]
    }

    pub fn into_linear(number: f32) -> f32 {
        number.powf(2.2)
    }
}

impl From<[f32; 3]> for Color {
    fn from(w: [f32; 3]) -> Color {
        Color {
            r: w[0],
            b: w[1],
            g: w[2],
        }
    }
}
impl From<Color> for [f32; 3] {
    fn from(w: Color) -> [f32; 3] {
        [w.r, w.b, w.g]
    }
}
